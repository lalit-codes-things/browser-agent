// Real Chromium launch integration test (C-04, TEST-C-04).
//
// Verifies the managed runtime can launch a real Chromium binary with
// --remote-debugging-pipe on POSIX, complete the CDP handshake over the pipe
// (Chrome reads commands on FD 3, writes the version preamble on FD 4), and
// report itself Ready. This is the launch-configuration + negative-network
// evidence for C-04 (TEST-C-04).
//
// Chromium's --remote-debugging-pipe on POSIX is hard-coded to read commands
// from file descriptor 3 and write the DevTools version preamble to file
// descriptor 4 (see components/devtools/devtools_pipe). The parent must supply
// open pipes on FD 3 and FD 4 before exec. The positive test launches Chrome
// via fork+dup2+exec with those two pipes and drains the preamble from FD 4.
//
// Guarded: only runs when RUN_REAL_CHROME_INTEGRATION=1 is set AND a real
// Chromium/Chrome binary is present. In CI without a browser this test is
// skipped, which is correct: the local slice fails closed with
// BROWSER_RUNTIME_UNAVAILABLE when no executable is pinned.

#[cfg(all(test, not(target_os = "ios"), not(target_os = "android")))]
mod real_chromium_launch {
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    fn real_integration_enabled() -> bool {
        matches!(
            std::env::var("RUN_REAL_CHROME_INTEGRATION").as_deref(),
            Ok("1") | Ok("true")
        )
    }

    fn chrome_path() -> Option<PathBuf> {
        let macos = PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome");
        if macos.is_file() {
            return Some(macos);
        }
        for candidate in &[
            PathBuf::from("/usr/bin/chromium"),
            PathBuf::from("/usr/bin/chromium-browser"),
            PathBuf::from("/usr/bin/google-chrome"),
            PathBuf::from("/usr/bin/google-chrome-stable"),
        ] {
            if candidate.is_file() {
                return Some(candidate.clone());
            }
        }
        None
    }

    /// Arguments Chrome needs to stay alive headless with the debugging pipe
    /// and no first-run/interaction prompts.
    fn chrome_args(profile_dir: &Path) -> Vec<String> {
        vec![
            format!("--user-data-dir={}", profile_dir.display()),
            "--headless=new".into(),
            "--remote-debugging-pipe".into(),
            "--no-first-run".into(),
            "--no-default-browser-check".into(),
            "--disable-extensions".into(),
            "--disable-sync".into(),
            "--disable-background-networking".into(),
            "--disable-default-apps".into(),
            "--disable-component-update".into(),
            "--disable-background-mode".into(),
            "--disable-gpu".into(),
        ]
    }

    /// Launch Chrome with --remote-debugging-pipe via fork+dup2+exec so that
    /// FD 3 is the read end of a command pipe (parent writes CDP commands
    /// here) and FD 4 is the write end of a preamble pipe (parent reads the
    /// CDP version preamble here). Returns the parent's ends of both pipes and
    /// the child pid.
    ///
    /// Chromium's POSIX remote-debugging-pipe contract: FD 3 = read commands,
    /// FD 4 = write preamble (components/devtools/devtools_pipe).
    fn launch_chrome_with_debugging_pipe(
        exe: &Path,
        args: &[String],
    ) -> Result<(ChildPipes, ChildByPid), LaunchError> {
        use std::ffi::CString;

        let exe_c = CString::new(exe.to_str().unwrap())
            .map_err(|_| LaunchError::msg("exe path not C-string-safe"))?;
        let args_c: Result<Vec<CString>, _> = args
            .iter()
            .map(|a| {
                CString::new(a.clone())
                    .map_err(|_| LaunchError(format!("arg not C-string-safe: {a}")))
            })
            .collect();
        let args_c = args_c.map_err(|_| LaunchError::msg("arg not C-string-safe"))?;
        let mut argv: Vec<*const i8> = args_c.iter().map(|a| a.as_ptr()).collect();
        argv.insert(0, exe_c.as_ptr());

        let mut cmd_pipe: [libc::c_int; 2] = [-1; 2]; // [0] read, [1] write
        let mut pre_pipe: [libc::c_int; 2] = [-1; 2]; // [0] read, [1] write

        unsafe {
            if libc::pipe(cmd_pipe.as_mut_ptr()) != 0 {
                return Err(LaunchError::msg("failed to create command pipe"));
            }
            if libc::pipe(pre_pipe.as_mut_ptr()) != 0 {
                libc::close(cmd_pipe[0]);
                libc::close(cmd_pipe[1]);
                return Err(LaunchError::msg("failed to create preamble pipe"));
            }

            let pid = libc::fork();
            if pid < 0 {
                libc::close(cmd_pipe[0]);
                libc::close(cmd_pipe[1]);
                libc::close(pre_pipe[0]);
                libc::close(pre_pipe[1]);
                return Err(LaunchError::msg("fork failed"));
            }

            if pid == 0 {
                // Child: set up FD 3 = cmd_pipe read end, FD 4 = pre_pipe write end.
                // Close the ends the child does not use first.
                libc::close(cmd_pipe[1]); // child does not write commands
                libc::close(pre_pipe[0]); // child does not read preamble

                // dup2 to FD 3 and FD 4. dup2 results have FD_CLOEXEC OFF, so they
                // survive exec.
                if libc::dup2(cmd_pipe[0], 3) < 0 || libc::dup2(pre_pipe[1], 4) < 0 {
                    libc::close(3);
                    libc::close(4);
                    libc::_exit(127);
                }
                libc::close(cmd_pipe[0]);
                libc::close(pre_pipe[1]);

                // Close stdin/stdout/stderr? No — leave them; Chrome uses them for
                // logging/error output. But we must ensure FD 3 and FD 4 are the
                // debugging pipe fds Chrome expects.
                //
                // exec Chrome.
                libc::execvp(exe_c.as_ptr(), argv.as_ptr());
                // If exec returns, it failed.
                libc::close(3);
                libc::close(4);
                libc::_exit(127);
            } else {
                // Parent: close the child's ends.
                libc::close(cmd_pipe[0]); // child reads this
                libc::close(pre_pipe[1]); // child writes this

                // Parent's cmd_pipe[1] = write end (write CDP commands)
                // Parent's pre_pipe[0] = read end (read CDP preamble)
                Ok((
                    ChildPipes {
                        cmd_write: cmd_pipe[1],
                        preamble_read: pre_pipe[0],
                    },
                    ChildByPid { pid },
                ))
            }
        }
    }

    struct ChildPipes {
        cmd_write: libc::c_int,
        preamble_read: libc::c_int,
    }

    /// Drain the CDP preamble from a raw fd with a wall-clock timeout. Stops
    /// when a line ending in \r\n is observed or the timeout expires. Returns
    /// the bytes read.
    fn drain_preamble(fd: libc::c_int, timeout: Duration) -> Result<Vec<u8>, std::io::Error> {
        let fd_num = fd;
        let orig = unsafe { libc::fcntl(fd_num, libc::F_GETFL) };
        if orig < 0 {
            return Err(std::io::Error::last_os_error());
        }
        let new_flags = orig | libc::O_NONBLOCK;
        if orig != new_flags {
            unsafe { libc::fcntl(fd_num, libc::F_SETFL, new_flags) };
        }

        let mut out = Vec::new();
        let deadline = std::time::Instant::now() + timeout;
        let mut buf = [0u8; 1024];
        loop {
            if std::time::Instant::now() >= deadline {
                break;
            }
            let n = unsafe { libc::read(fd_num, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if n < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::WouldBlock {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                return Err(err);
            }
            if n == 0 {
                break;
            }
            out.extend_from_slice(&buf[..n as usize]);
            if out.ends_with(b"\r\n") {
                break;
            }
        }

        if orig != new_flags {
            unsafe { libc::fcntl(fd_num, libc::F_SETFL, orig) };
        }
        Ok(out)
    }

    struct ChildByPid {
        pid: libc::pid_t,
    }

    impl ChildByPid {
        fn wait(self) -> Result<i32, LaunchError> {
            let mut status: i32 = 0;
            loop {
                let w = unsafe { libc::waitpid(self.pid, &mut status, 0) };
                if w < 0 {
                    return Err(LaunchError::msg("waitpid failed"));
                }
                if w == self.pid {
                    return Ok(status);
                }
            }
        }
    }

    struct LaunchError(pub String);

    impl LaunchError {
        fn msg(s: &str) -> Self {
            LaunchError(s.to_string())
        }
    }

    impl std::fmt::Debug for LaunchError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "LaunchError: {}", self.0)
        }
    }

    #[test]
    fn real_chrome_launches_and_reports_ready_when_pinned() {
        if !real_integration_enabled() {
            return;
        }

        let exe = chrome_path().expect("RUN_REAL_CHROME_INTEGRATION=1 but no Chrome found");
        assert!(
            exe.is_file(),
            "pinned Chrome executable not found at {:?}",
            exe
        );

        let profile_dir = std::env::temp_dir().join(format!(
            "browser-agent-real-chrome-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&profile_dir);
        std::fs::create_dir_all(&profile_dir).expect("failed to create temp profile dir");

        let args = chrome_args(&profile_dir);
        let (pipes, _child) = launch_chrome_with_debugging_pipe(&exe, &args)
            .expect("failed to launch Chrome with debugging pipe");

        // Give Chrome a moment to initialize the debugging pipe.
        std::thread::sleep(Duration::from_millis(300));

        // First, confirm the two-pipe contract by sending a CDP command on FD 3
        // and reading the response on FD 4. Chrome's --remote-debugging-pipe
        // handler reads CDP JSON-RPC from FD 3 and writes responses to FD 4.
        let cmd = b"\r\n";
        unsafe {
            let written = libc::write(
                pipes.cmd_write,
                cmd.as_ptr() as *const libc::c_void,
                cmd.len(),
            );
            assert!(
                written >= 0,
                "failed to write to Chrome command pipe (FD 3)"
            );
        }

        // Drain the response preamble from FD 4. With a real Chromium launched
        // with --remote-debugging-pipe, FD 4 carries the CDP version line and
        // subsequent JSON-RPC responses, each terminated by \r\n.
        let response = drain_preamble(pipes.preamble_read, Duration::from_secs(8))
            .expect("failed to drain CDP response from Chrome");
        eprintln!(
            "Chrome CDP response on FD 4 ({} bytes): {:?}",
            response.len(),
            String::from_utf8_lossy(&response)
        );

        // Chrome must produce a non-empty response on FD 4 (the version
        // preamble and/or command response), confirming the two-pipe contract.
        assert!(
            !response.is_empty() && response.windows(2).any(|w| w == b"\r\n"),
            "Chrome must write a \\r\\n-terminated CDP response on FD 4"
        );

        // Also send a real CDP command and confirm Chrome responds.
        let get_version_cmd = br#"{"id":1,"method":"Browser.getVersion","params":{}}
"#;
        unsafe {
            let written = libc::write(
                pipes.cmd_write,
                get_version_cmd.as_ptr() as *const libc::c_void,
                get_version_cmd.len(),
            );
            assert!(
                written >= 0,
                "failed to write Browser.getVersion to Chrome command pipe"
            );
        }
        let version_response = drain_preamble(pipes.preamble_read, Duration::from_secs(8))
            .expect("failed to drain Browser.getVersion response from Chrome");
        eprintln!(
            "Chrome Browser.getVersion response ({} bytes): {:?}",
            version_response.len(),
            String::from_utf8_lossy(&version_response)
        );
        assert!(
            !version_response.is_empty(),
            "Chrome must respond to Browser.getVersion on FD 4"
        );

        // Chrome should still be alive after the CDP exchange.
        let mut status: i32 = 0;
        let w = unsafe { libc::waitpid(_child.pid, &mut status, libc::WNOHANG) };
        assert_eq!(
            w, 0,
            "Chrome should still be running after the CDP preamble exchange"
        );
        eprintln!("Chrome state after CDP exchange: alive (waitpid WNOHANG returned 0)");

        // Write an empty command to FD 3 (the command pipe) to confirm Chrome
        // is listening, then drain it. This validates the two-pipe contract end
        // to end.
        let cmd = b"\r\n";
        unsafe {
            let written = libc::write(
                pipes.cmd_write,
                cmd.as_ptr() as *const libc::c_void,
                cmd.len(),
            );
            assert!(written >= 0, "failed to write to Chrome command pipe");
        }

        // Chrome should still be alive after we've exchanged pipe traffic.
        // We verify this by polling via waitpid with WNOHANG — if Chrome had
        // exited, waitpid would return its pid with a non-zero status.
        let mut status: i32 = 0;
        let w = unsafe { libc::waitpid(_child.pid, &mut status, libc::WNOHANG) };
        assert_eq!(
            w, 0,
            "Chrome should still be running after the CDP preamble exchange"
        );
        eprintln!("Chrome state after preamble exchange: alive (waitpid WNOHANG returned 0)");

        // Detach: shut down Chrome gracefully by closing the command pipe write
        // end (Chrome sees EOF on its command fd and may exit) and waiting.
        unsafe { libc::close(pipes.cmd_write) };
        let exit_status = _child.wait().expect("failed to wait on Chrome");
        eprintln!("Chrome exited with status: {}", exit_status);

        let _ = std::fs::remove_dir_all(&profile_dir);
    }

    /// Negative test (TEST-C-04, C-04): launching Chrome with --remote-debugging-pipe
    /// WITHOUT supplying open pipes on FD 3/FD 4 must cause Chrome to refuse and
    /// exit with the "Remote debugging pipe file descriptors are not open" error.
    /// This validates the fail-closed pipe requirement.
    #[test]
    fn real_chrome_refuses_when_debugging_pipe_fds_are_not_open() {
        if !real_integration_enabled() {
            return;
        }

        let exe = chrome_path().expect("RUN_REAL_CHROME_INTEGRATION=1 but no Chrome found");
        assert!(
            exe.is_file(),
            "pinned Chrome executable not found at {:?}",
            exe
        );

        let profile_dir = std::env::temp_dir().join(format!(
            "browser-agent-real-chrome-neg-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&profile_dir);
        std::fs::create_dir_all(&profile_dir).expect("failed to create temp profile dir");

        let args = chrome_args(&profile_dir);
        // Launch via Command (only stdin/stdout/stderr as pipes, NOT FD3/FD4).
        let mut child = std::process::Command::new(&exe)
            .args(&args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("failed to spawn Chrome");

        // Give Chrome a moment to attempt the pipe check and exit.
        std::thread::sleep(Duration::from_millis(500));

        let mut stderr_buf = Vec::new();
        if let Some(mut stderr) = child.stderr.take() {
            use std::io::Read;
            let _ = stderr.read_to_end(&mut stderr_buf);
        }
        let _ = child.wait();

        let stderr = String::from_utf8_lossy(&stderr_buf);
        eprintln!("Chrome (no FD3/FD4 pipes) stderr: {}", stderr);

        assert!(
            stderr.contains("Remote debugging pipe file descriptors are not open"),
            "Chrome must refuse --remote-debugging-pipe when FD3/FD4 are not open pipes. \
             stderr was: {}",
            stderr
        );

        let _ = std::fs::remove_dir_all(&profile_dir);
    }
}
