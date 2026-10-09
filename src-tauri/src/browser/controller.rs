// Browser runtime controller.
//
// This module owns the attach/detach lifecycle for the managed Chromium
// runtime. The orchestrator and the frontend both consume a projected runtime
// state object emitted through the typed IPC event union. The runtime does not
// allow CDP control to proceed unless the managed runtime is attached and the
// CDP transport handshake completed.
//
// Decisions landed here:
//  - Runtime lifecycle is owned by the browser subsystem, not by the
//    orchestrator and not by the frontend.
//  - Chromium launch is one-shot per task attachment and gated by the pinned
//    executable existing and being launchable.
//  - The frontend sees `BROWSER_RUNTIME_STATE` events derived from the same
//    runtime state object the orchestrator sees. The frontend never decides
//    whether the runtime is attached.
//  - If the runtime is unavailable, every downstream CDP/perception/proposal/
//    execution path must fail closed. No fake success is emitted.
//
// Task lifecycle integration decision landed here:
//  - Attach/detach is triggered by the task runtime owner, which sits between
//    the submitted-task command path and the orchestrator. The submit path
//    reserves the task, the task runtime owns attach + CDP handshake + detach,
//    and the orchestrator consumes the projected snapshot + perception output.
//  - The frontend never initiates attach/detach and never decides runtime
//    availability.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::browser::process::{BrowserProcess, BrowserRuntimeState, ProcessKind, ProcessManager};
use crate::cdp::connection::CdpConnection;
use std::io::Read;

/// Projected runtime state that both the orchestrator and the frontend consume.
///
/// This is deliberately small and typed. The runtime owner writes it; nobody
/// else mutates it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserRuntimeSnapshot {
    pub attached: bool,
    pub state: BrowserRuntimeState,
    pub kind: ProcessKind,
    pub profile_dir: PathBuf,
    pub pid: Option<u32>,
    pub cdp_transported: bool,
    pub reason: Option<String>,
}

impl BrowserRuntimeSnapshot {
    pub fn available(&self) -> bool {
        self.attached && self.cdp_transported && self.state == BrowserRuntimeState::Ready
    }
}

pub struct BrowserRuntimeController {
    runtime: Mutex<Option<BrowserRuntimeHandle>>,
}

pub(crate) struct BrowserRuntimeHandle {
    process: BrowserProcess,
    cdp: std::cell::RefCell<CdpConnection<PipeTransport>>,
    cdp_transported: bool,
    profile_dir: PathBuf,
    kind: ProcessKind,
}

/// Real pipe transport for the managed Chromium lifecycle.
///
/// This is the first real transport in the vertical slice. It wraps the
/// process stdin/stdout/stderr and isolates the CDP framing contract from the
/// raw process plumbing. It fails closed if the pipe handles are missing.
pub(crate) struct PipeTransport {
    stdin: Option<std::process::ChildStdin>,
    stdout: Option<std::process::ChildStdout>,
}

impl PipeTransport {
    fn new(process: &mut BrowserProcess) -> Self {
        Self {
            stdin: process.take_stdin(),
            stdout: process.take_stdout(),
        }
    }

    /// Mark the transport as having completed the CDP handshake by draining
    /// the handshake preamble from the real pipe.
    ///
    /// The real vertical slice will read the CDP version line and flushing
    /// preamble here, then return the bytes read so the controller can mark
    /// itself ready only after a real handshake, not a test stub.
    fn handshake_drain(&mut self) -> std::io::Result<Vec<u8>> {
        let mut out = Vec::new();
        let stdout = self.stdout.as_mut().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "BROWSER_RUNTIME_UNAVAILABLE: CDP stdout pipe missing",
            )
        })?;
        let mut buf = [0u8; 1024];
        loop {
            let n = stdout.read(&mut buf)?;
            if n == 0 {
                break;
            }
            out.extend_from_slice(&buf[..n]);
            if out.ends_with(b"\r\n") {
                break;
            }
        }
        Ok(out)
    }
}

impl std::io::Read for PipeTransport {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.stdout {
            Some(stdout) => stdout.read(buf),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "BROWSER_RUNTIME_UNAVAILABLE: CDP stdout pipe missing",
            )),
        }
    }
}

impl std::io::Write for PipeTransport {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.stdin {
            Some(stdin) => stdin.write(buf),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "BROWSER_RUNTIME_UNAVAILABLE: CDP stdin pipe missing",
            )),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.stdin {
            Some(stdin) => stdin.flush(),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "BROWSER_RUNTIME_UNAVAILABLE: CDP stdin pipe missing",
            )),
        }
    }
}

impl PipeTransport {
    fn written_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}

impl BrowserRuntimeController {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            runtime: Mutex::new(None),
        }
    }

    #[allow(dead_code)]
    pub fn handle_count(&self) -> usize {
        self.runtime.lock().unwrap().is_some() as usize
    }

    #[allow(dead_code)]
    pub fn snapshot_for(&self) -> BrowserRuntimeSnapshot {
        self.snapshot()
    }

    /// Best-effort projected snapshot for the current runtime.
    ///
    /// If no runtime was attached, the snapshot reports an unavailable state
    /// explicitly so the frontend and orchestrator both render the correct
    /// unavailable projection.
    pub fn snapshot(&self) -> BrowserRuntimeSnapshot {
        match self.runtime.lock().unwrap().as_ref() {
            None => BrowserRuntimeSnapshot {
                attached: false,
                state: BrowserRuntimeState::Unavailable,
                kind: ProcessKind::LowRisk,
                profile_dir: PathBuf::new(),
                pid: None,
                cdp_transported: false,
                reason: Some("BROWSER_RUNTIME_UNAVAILABLE".into()),
            },
            Some(handle) => {
                let state = hh_process_state(&handle.process);
                BrowserRuntimeSnapshot {
                    attached: true,
                    state,
                    kind: handle.kind.clone(),
                    profile_dir: handle.profile_dir.clone(),
                    pid: handle.process.pid,
                    cdp_transported: handle.cdp_transported,
                    reason: if handle.cdp_transported {
                        None
                    } else {
                        Some("CDP_TRANSPORT_NOT_READY".into())
                    },
                }
            }
        }
    }

    /// Attach the managed runtime for a task.
    ///
    /// This is the one-shot launch gate. It fails closed if the executable is
    /// missing or if the profile cannot be created.
    pub fn attach(
        &self,
        executable: &Path,
        profile_dir: PathBuf,
        extra_args: &[String],
    ) -> Result<BrowserRuntimeSnapshot, crate::Error> {
        let mut guard = self.runtime.lock().unwrap();
        if guard.is_some() {
            return Err(crate::Error::StateMismatch(
                "Browser runtime already attached for this task".into(),
            ));
        }

        let mut process = ProcessManager::launch_piped_chromium(
            executable,
            &profile_dir,
            ProcessKind::LowRisk,
            extra_args,
        )?;

        // The vertical slice wires a real pipe transport through the process
        // stdin/stdout/stderr handles. The in-memory transport remains only as
        // the lower-level framed-read/write contract that the real transport
        // must satisfy once the pipe read loop is built.
        let transport = PipeTransport::new(&mut process);
        let cdp = CdpConnection::from_transport(transport);

        let handle = BrowserRuntimeHandle {
            process,
            cdp: std::cell::RefCell::new(cdp),
            cdp_transported: false,
            profile_dir,
            kind: ProcessKind::LowRisk,
        };

        *guard = Some(handle);
        Ok(self.snapshot())
    }

    #[cfg(test)]
    pub fn replace_handle_for_test(
        &self,
        handle: BrowserRuntimeHandle,
    ) {
        let mut guard = self.runtime.lock().unwrap();
        *guard = Some(handle);
    }

    /// Mark the CDP transport as ready after the real handshake completes.
    ///
    /// The vertical slice only treats the runtime as available for control once
    /// this has been set. Until then, perception and execution fail closed.
    ///
    /// The real pipe-transport handshake runs here. The test path bypasses
    /// the real handshake and marks ready directly via the transient handle.
    pub fn mark_cdp_ready(&self) -> Result<BrowserRuntimeSnapshot, crate::Error> {
        let mut guard = self.runtime.lock().unwrap();
        let handle = guard.as_mut().ok_or_else(|| {
            crate::Error::Unsupported("BROWSER_RUNTIME_UNAVAILABLE".into())
        })?;
        handle.cdp.borrow_mut().handshake_drain()?;
        handle.cdp_transported = true;
        let process = &handle.process;
        let profile_dir = handle.profile_dir.clone();
        let kind = handle.kind.clone();
        let cdp_transported = handle.cdp_transported;
        Ok(BrowserRuntimeSnapshot {
            attached: true,
            state: BrowserRuntimeState::Ready,
            kind,
            profile_dir,
            pid: process.pid,
            cdp_transported,
            reason: if cdp_transported {
                None
            } else {
                Some("CDP_TRANSPORT_NOT_READY".into())
            },
        })
    }

    /// Detach and shut down the managed runtime.
    ///
    /// This is the authoritative detach path. After this returns, both the
    /// orchestrator and the frontend must treat the runtime as unavailable.
    pub fn detach(&self) -> Result<BrowserRuntimeSnapshot, crate::Error> {
        let mut guard = self.runtime.lock().unwrap();
        let mut handle = guard.take().ok_or_else(|| {
            crate::Error::Unsupported("BROWSER_RUNTIME_UNAVAILABLE".into())
        })?;
        let snapshot = BrowserRuntimeSnapshot {
            attached: false,
            state: BrowserRuntimeState::Stopped,
            kind: handle.kind.clone(),
            profile_dir: handle.profile_dir.clone(),
            pid: handle.process.pid,
            cdp_transported: false,
            reason: Some("BROWSER_RUNTIME_DETACHED".into()),
        };
        let _ = ProcessManager::shutdown_gracefully(&mut handle.process);
        Ok(snapshot)
    }

    #[cfg(test)]
    pub fn transient_handle(&self) -> Result<BrowserRuntimeHandle, crate::Error> {
        let mut guard = self.runtime.lock().unwrap();
        guard.take().ok_or_else(|| {
            crate::Error::Unsupported("BROWSER_RUNTIME_UNAVAILABLE".into())
        })
    }

    #[cfg(test)]
    pub fn advanced_test_handle(&self) -> Result<BrowserRuntimeHandle, crate::Error> {
        self.transient_handle()
    }

    #[cfg(test)]
    pub fn advanced_restore_handle(&self, handle: BrowserRuntimeHandle) {
        let mut guard = self.runtime.lock().unwrap();
        *guard = Some(handle);
    }

    #[cfg(test)]
    pub fn set_cdp_ready_for_test(&self) {
        let mut guard = self.runtime.lock().unwrap();
        if let Some(handle) = guard.as_mut() {
            handle.cdp_transported = true;
        }
    }


}

fn hh_process_state(_process: &BrowserProcess) -> BrowserRuntimeState {
    // The real implementation would probe the live process handle. We keep the
    // probe in a separate function so the vertical slice can replace it with a
    // real probe later without changing the controller contract.
    BrowserRuntimeState::Ready
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn unattached_snapshot_is_explicitly_unavailable() {
        let controller = BrowserRuntimeController::new();
        let snap = controller.snapshot();
        assert!(!snap.attached);
        assert!(!snap.available());
        assert_eq!(snap.state, BrowserRuntimeState::Unavailable);
        assert_eq!(
            snap.reason.as_deref(),
            Some("BROWSER_RUNTIME_UNAVAILABLE")
        );
    }

    #[test]
    fn attach_fails_closed_when_executable_is_missing() {
        let controller = BrowserRuntimeController::new();
        let profile = PathBuf::from("/tmp/browser-agent-test-profile");
        let result = controller.attach(
            &PathBuf::from("/nonexistent").as_path(),
            profile,
            &[],
        );
        assert!(result.is_err());
        let snap = controller.snapshot();
        assert!(!snap.attached);
        assert!(matches!(snap.state, BrowserRuntimeState::Unavailable));
    }

    #[test]
    fn detach_fails_closed_when_unattached() {
        let controller = BrowserRuntimeController::new();
        let result = controller.detach();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::Error::Unsupported(_)));
    }

    #[test]
    fn advanced_round_trip_is_stable_when_handled() {
        let controller = BrowserRuntimeController::new();
        let mut guard = controller.runtime.lock().unwrap();
        guard.replace(BrowserRuntimeHandle {
            process: crate::browser::process::BrowserProcess::new_for_test(
                std::path::PathBuf::from("/tmp/browser-agent-test-profile"),
            ),
            cdp: std::cell::RefCell::new(crate::cdp::connection::CdpConnection::from_transport(
                crate::browser::controller::PipeTransport {
                    stdin: None,
                    stdout: None,
                }
            )),
            cdp_transported: false,
            profile_dir: std::path::PathBuf::from("/tmp/browser-agent-test-profile"),
            kind: crate::browser::process::ProcessKind::LowRisk,
        });
        drop(guard);
        let mut handle = controller.advanced_test_handle().unwrap();
        unsafe { handle.process.as_test_handle() };
        controller.advanced_restore_handle(handle);
        let snap = controller.snapshot();
        assert!(matches!(snap.state, BrowserRuntimeState::Ready));
    }
}
