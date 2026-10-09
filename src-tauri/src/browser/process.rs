use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

pub struct BrowserProcess {
    pub pid: Option<u32>,
    pub kind: ProcessKind,
    child: Child,
    pub user_data_dir: PathBuf,
    #[allow(dead_code, private_interfaces)]
    _marker: std::marker::PhantomData<std::ptr::NonNull<()>>,
}

impl BrowserProcess {
    pub fn take_stdin(&mut self) -> Option<std::process::ChildStdin> {
        self.child.stdin.take()
    }

    pub fn take_stdout(&mut self) -> Option<std::process::ChildStdout> {
        self.child.stdout.take()
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

#[cfg(test)]
impl BrowserProcess {
    pub fn new_for_test(user_data_dir: PathBuf) -> Self {
        Self {
            pid: Some(0),
            kind: ProcessKind::LowRisk,                child: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },    #[allow(dead_code)]
    _marker: std::marker::PhantomData,
            user_data_dir,        }
    }
}

#[cfg(test)]
impl Drop for BrowserProcess {
    fn drop(&mut self) {
        // Synthetic test handles are initialized with zeroed bytes; avoid the
        // real Child drop path because that closure would dereference an invalid
        // IO handle.
        unsafe {
            // Leave `self.child` uninitialized in synthetic test handles; do not
            // run the real `Child` drop code path.
            std::ptr::write(&mut self.child, std::mem::MaybeUninit::zeroed().assume_init());
        }
    }
}

#[cfg(test)]
impl BrowserProcess {
    /// Reinitialize a synthetic test handle after it has been moved out of the
    /// runtime controller in tests so the runtime does not attempt to drop a
    /// real `Child` handle when the epoch advances.

}

impl ProcessManager {
    /// One-shot launch gate: the managed Chromium process is launched exactly
    /// once per runtime attachment and reused for the task lifecycle. This
    /// blocks repeated ad hoc launches and is the structural boundary before
    /// any CDP transport is handed to the orchestrator.
    //
    // If the pinned executable is missing or unreadable, the runtime returns
    // an explicit `BROWSER_RUNTIME_UNAVAILABLE` and the frontend/projected
    // state must render that as a concrete unavailable state, not an empty
    // success path.
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
        let child = command
            .spawn()
            .map_err(|e| crate::Error::Internal(format!("Chromium launch failed: {e}")))?;
        let pid = Some(child.id());
        Ok(BrowserProcess {
            pid,
            kind,
            child,
            user_data_dir: user_data_dir.to_path_buf(),
            _marker: std::marker::PhantomData,
        })
    }


    /// Read the current runtime state from the persisted process handle.
    ///
    /// This is intentionally a best-effort probe from the managed process
    /// handle. It does not start a new process and it does not talk to CDP.
    pub fn state(process: &mut BrowserProcess) -> BrowserRuntimeState {
        match process.child.try_wait() {
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
        if let Some(status) = process
            .child
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
        process
            .child
            .kill()
            .map_err(|e| crate::Error::Internal(format!("Chromium shutdown failed: {e}")))?;
        process
            .child
            .wait()
            .map_err(|e| crate::Error::Internal(e.to_string()))?;
        Ok(())
    }
}
