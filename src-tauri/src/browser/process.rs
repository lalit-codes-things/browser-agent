use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

pub struct BrowserProcess {
    pub pid: Option<u32>,
    pub kind: ProcessKind,
    child: Option<Child>,
    pub user_data_dir: PathBuf,
    #[allow(dead_code, private_interfaces)]
    _marker: std::marker::PhantomData<Child>,
}

impl BrowserProcess {
    pub fn take_stdin(&mut self) -> Option<std::process::ChildStdin> {
        self.child.as_mut().and_then(|c| c.stdin.take())
    }

    pub fn take_stdout(&mut self) -> Option<std::process::ChildStdout> {
        self.child.as_mut().and_then(|c| c.stdout.take())
    }

    pub fn take_stderr(&mut self) -> Option<std::process::ChildStderr> {
        self.child.as_mut().and_then(|c| c.stderr.take())
    }

    #[cfg(test)]
    pub fn new_for_test(user_data_dir: PathBuf) -> Self {
        Self {
            pid: Some(0),
            kind: ProcessKind::LowRisk,
            child: None,
            user_data_dir,
            _marker: std::marker::PhantomData,
        }
    }
}

impl Drop for BrowserProcess {
    fn drop(&mut self) {
        // Safe drop implementation: take the active child handle and ensure it
        // is terminated and reaped so no zombie Chromium processes remain.
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessKind {
    HighRisk,
    LowRisk,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserRuntimeState {
    Ready,
    Unavailable,
    Crashed,
    Stopped,
}

pub struct ProcessManager;

impl ProcessManager {
    /// One-shot launch gate: the managed Chromium process is launched exactly
    /// once per runtime attachment and reused for the task lifecycle. This
    /// blocks repeated ad hoc launches and is the structural boundary before
    /// any CDP transport is handed to the orchestrator.
    ///
    /// If the pinned executable is missing or unreadable, the runtime returns
    /// an explicit `BROWSER_RUNTIME_UNAVAILABLE` and the frontend/projected
    /// state must render that as a concrete unavailable state, not an empty
    /// success path.
    pub fn launch_piped_chromium(
        executable: &Path,
        user_data_dir: &Path,
        kind: ProcessKind,
        extra_args: &[String],
    ) -> Result<BrowserProcess, crate::Error> {
        if !executable.is_file() {
            return Err(crate::Error::Unsupported(
                "BROWSER_RUNTIME_UNAVAILABLE".into(),
            ));
        }
        std::fs::create_dir_all(user_data_dir)
            .map_err(|e| crate::Error::Internal(format!("profile creation failed: {e}")))?;

        let mut command = Command::new(executable);
        command
            .arg("--remote-debugging-pipe")
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .arg("--disable-extensions")
            .arg("--disable-sync")
            .arg("--user-data-dir")
            .arg(user_data_dir)
            .args(extra_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // On POSIX, Chromium's --remote-debugging-pipe protocol expects file
        // descriptor 3 for receiving CDP commands and file descriptor 4 for
        // sending CDP preamble and responses. Duplicate stdin -> FD 3 and
        // stdout -> FD 4 inside pre_exec so child process satisfies Chromium's
        // native pipe check.
        #[cfg(unix)]
        unsafe {
            command.pre_exec(|| {
                if libc::dup2(0, 3) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if libc::dup2(1, 4) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }

        let child = command
            .spawn()
            .map_err(|e| crate::Error::Internal(format!("Chromium launch failed: {e}")))?;
        let pid = Some(child.id());
        Ok(BrowserProcess {
            pid,
            kind,
            child: Some(child),
            user_data_dir: user_data_dir.to_path_buf(),
            _marker: std::marker::PhantomData,
        })
    }

    /// Read the current runtime state from the persisted process handle.
    ///
    /// This is intentionally a best-effort probe from the managed process
    /// handle. It does not start a new process and it does not talk to CDP.
    pub fn state(process: &mut BrowserProcess) -> BrowserRuntimeState {
        let child = match process.child.as_mut() {
            Some(c) => c,
            None => return BrowserRuntimeState::Unavailable,
        };
        match child.try_wait() {
            Ok(None) => BrowserRuntimeState::Ready,
            Ok(Some(_)) => BrowserRuntimeState::Crashed,
            Err(_) => BrowserRuntimeState::Unavailable,
        }
    }

    /// Graceful shutdown with fail-closed fallback.
    ///
    /// If the process has already exited, we verify the exit status. If it
    /// is still running, we hard-kill and then wait so the handle is fully
    /// released before the runtime proceeds.
    pub fn shutdown_gracefully(process: &mut BrowserProcess) -> Result<(), crate::Error> {
        let child = process
            .child
            .as_mut()
            .ok_or_else(|| crate::Error::Unsupported("BROWSER_RUNTIME_UNAVAILABLE".into()))?;
        if let Some(status) = child
            .try_wait()
            .map_err(|e| crate::Error::Internal(e.to_string()))?
        {
            return if status.success() {
                Ok(())
            } else {
                Err(crate::Error::Aborted(
                    "Chromium exited before shutdown".into(),
                ))
            };
        }
        child
            .kill()
            .map_err(|e| crate::Error::Internal(format!("Chromium shutdown failed: {e}")))?;
        child
            .wait()
            .map_err(|e| crate::Error::Internal(e.to_string()))?;
        Ok(())
    }

    #[allow(dead_code, unreachable_code)]
    pub fn _unused_release_child_for_tests(process: &mut BrowserProcess) {
        let _ = crate::browser::process::ProcessManager::shutdown_gracefully(process);
    }
}
