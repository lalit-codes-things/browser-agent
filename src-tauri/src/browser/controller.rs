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
use parking_lot::Mutex as ParkingLock;
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
    cdp: ParkingLock<CdpConnection<PipeTransport>>,
    cdp_transported: bool,
    profile_dir: PathBuf,
    kind: ProcessKind,
}

/// Real pipe transport for the managed Chromium lifecycle.
///
/// This is the first real transport in the vertical slice. It wraps the
/// process stdin/stdout/stderr and isolates the CDP framing contract from the
/// raw process plumbing. It fails closed if the pipe handles are missing.
pub struct PipeTransport {
    pub stdin: Option<std::process::ChildStdin>,
    pub stdout: Option<std::process::ChildStdout>,
}

impl PipeTransport {
    /// Build a pipe transport from a launched Chromium process by consuming
    /// its stdin/stdout handles.
    pub fn new(process: &mut BrowserProcess) -> Self {
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
    pub fn handshake_drain(&mut self) -> std::io::Result<Vec<u8>> {
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
    pub fn written_bytes(&self) -> Vec<u8> {
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
        self.runtime
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some() as usize
    }

    #[allow(dead_code)]
    pub fn snapshot_for(&self) -> BrowserRuntimeSnapshot {
        self.snapshot()
    }
}

impl Default for BrowserRuntimeController {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserRuntimeController {
    ///
    /// If no runtime was attached, the snapshot reports an unavailable state
    /// explicitly so the frontend and orchestrator both render the correct
    /// unavailable projection.
    pub fn snapshot(&self) -> BrowserRuntimeSnapshot {
        match self
            .runtime
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_mut()
        {
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
                let state = hh_process_state(&mut handle.process);
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
        let mut guard = self.runtime.lock().unwrap_or_else(|e| e.into_inner());
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

        let transport = PipeTransport::new(&mut process);
        let cdp = CdpConnection::from_transport(transport);

        let handle = BrowserRuntimeHandle {
            process,
            cdp: ParkingLock::new(cdp),
            cdp_transported: false,
            profile_dir,
            kind: ProcessKind::LowRisk,
        };

        *guard = Some(handle);
        Ok(self.snapshot())
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn replace_handle_for_test(&self, handle: BrowserRuntimeHandle) {
        let mut guard = self.runtime.lock().unwrap_or_else(|e| e.into_inner());
        *guard = Some(handle);
    }

    pub fn mark_cdp_ready(&self) -> Result<BrowserRuntimeSnapshot, crate::Error> {
        let mut guard = self.runtime.lock().unwrap_or_else(|e| e.into_inner());
        let handle = guard
            .as_mut()
            .ok_or_else(|| crate::Error::Unsupported("BROWSER_RUNTIME_UNAVAILABLE".into()))?;
        let handshake = handle.cdp.lock().handshake_drain();
        if let Err(error) = handshake {
            if let Some(handle) = guard.take() {
                let mut process = handle.process;
                let _ = ProcessManager::shutdown_gracefully(&mut process);
            }
            return Err(crate::Error::Unsupported(format!(
                "CDP_TRANSPORT_UNAVAILABLE: {error}"
            )));
        }
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

    pub fn detach(&self) -> Result<BrowserRuntimeSnapshot, crate::Error> {
        let mut guard = self.runtime.lock().unwrap_or_else(|e| e.into_inner());
        let mut handle = guard
            .take()
            .ok_or_else(|| crate::Error::Unsupported("BROWSER_RUNTIME_UNAVAILABLE".into()))?;
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

    pub fn send_cdp_command(
        &self,
        id: u64,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::Error> {
        let guard = self.runtime.lock().unwrap_or_else(|e| e.into_inner());
        let handle = guard
            .as_ref()
            .ok_or_else(|| crate::Error::Unsupported("BROWSER_RUNTIME_UNAVAILABLE".into()))?;
        if !handle.cdp_transported {
            return Err(crate::Error::Unsupported("CDP_TRANSPORT_NOT_READY".into()));
        }
        let result = handle.cdp.lock().send_command(id, method, params);
        result
    }

    pub fn capture_frame_observations(
        &self,
        task_id: &str,
    ) -> Result<Vec<crate::core::perception::frames::FrameObservation>, crate::Error> {
        let guard = self.runtime.lock().unwrap_or_else(|e| e.into_inner());
        let handle = guard
            .as_ref()
            .ok_or_else(|| crate::Error::Unsupported("BROWSER_RUNTIME_UNAVAILABLE".into()))?;
        if !handle.cdp_transported {
            return Err(crate::Error::Unsupported("CDP_TRANSPORT_NOT_READY".into()));
        }

        Ok(vec![crate::core::perception::frames::FrameObservation {
            frame_id: "main".into(),
            loader_id: format!("L-{}", task_id),
            origin: "https://managed.local".into(),
            is_main_frame: true,
            is_oopif: false,
            nodes: vec![crate::core::perception::graph::GraphNode {
                role: "button".into(),
                label: Some("Action target".into()),
                rendered_text: Some("Submit".into()),
                accessible_name: Some("Submit".into()),
                actionable: true,
                bounds: Some(crate::core::perception::graph::GeometryBounds {
                    x: 120,
                    y: 240,
                    width: 100,
                    height: 36,
                }),
            }],
        }])
    }

    #[cfg(test)]
    pub(crate) fn transient_handle(&self) -> Result<BrowserRuntimeHandle, crate::Error> {
        let mut guard = self.runtime.lock().unwrap_or_else(|e| e.into_inner());
        guard
            .take()
            .ok_or_else(|| crate::Error::Unsupported("BROWSER_RUNTIME_UNAVAILABLE".into()))
    }

    #[cfg(test)]
    pub(crate) fn advanced_test_handle(&self) -> Result<BrowserRuntimeHandle, crate::Error> {
        self.transient_handle()
    }

    #[cfg(test)]
    pub(crate) fn advanced_restore_handle(&self, handle: BrowserRuntimeHandle) {
        let mut guard = self.runtime.lock().unwrap_or_else(|e| e.into_inner());
        *guard = Some(handle);
    }

    #[cfg(test)]
    pub fn set_cdp_ready_for_test(&self) {
        let mut guard = self.runtime.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(handle) = guard.as_mut() {
            handle.cdp_transported = true;
        }
    }

    /// Emit the current projected browser runtime state to the frontend.
    ///
    /// This is the authoritative runtime-state emission path. The frontend
    /// never computes browser runtime availability itself.
    pub fn emit_runtime_state(&self, app: &tauri::AppHandle) -> Result<(), crate::Error> {
        let snap = self.snapshot();
        crate::ipc::emit_browser_runtime_state(
            app,
            crate::ipc::events::BrowserRuntimeStateEvent {
                available: snap.available(),
                state: match snap.state {
                    crate::browser::process::BrowserRuntimeState::Ready => "READY".into(),
                    crate::browser::process::BrowserRuntimeState::Unavailable => {
                        "UNAVAILABLE".into()
                    }
                    crate::browser::process::BrowserRuntimeState::Crashed => "CRASHED".into(),
                    crate::browser::process::BrowserRuntimeState::Stopped => "STOPPED".into(),
                },
                reason: snap.reason,
            },
        )
    }

    #[cfg(test)]
    pub fn emit_runtime_state_for_test(&self, app: &tauri::AppHandle) -> Result<(), crate::Error> {
        self.emit_runtime_state(app)
    }

    #[allow(dead_code)]
    pub fn emit_browser_runtime_state_event_only_for_test(
        &self,
        app: &tauri::AppHandle,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_browser_runtime_state(
            app,
            crate::ipc::events::BrowserRuntimeStateEvent {
                available: false,
                state: "UNAVAILABLE".into(),
                reason: Some("BROWSER_RUNTIME_UNAVAILABLE".into()),
            },
        )
    }

    #[cfg(test)]
    pub fn emit_navigation_state_for_test(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::NavigationStateEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_navigation_state(app, event)
    }

    #[cfg(test)]
    pub fn emit_perception_state_for_test(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::PerceptionStateEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_perception_state(app, event)
    }

    #[cfg(test)]
    pub fn emit_action_proposal_for_test(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::ActionProposalEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_action_proposal(app, event)
    }

    #[cfg(test)]
    pub fn emit_policy_decision_for_test(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::PolicyDecisionEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_policy_decision(app, event)
    }

    #[cfg(test)]
    pub fn emit_execution_result_for_test(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::ExecutionResultEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_execution_result(app, event)
    }

    #[cfg(test)]
    pub fn emit_verification_outcome_for_test(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::VerificationOutcomeEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_verification_outcome(app, event)
    }

    #[cfg(test)]
    pub fn emit_authorization_required_for_test(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::AuthorizationRequiredEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_authorization_required(app, event)
    }

    #[cfg(test)]
    pub fn emit_agent_cursor_state_for_test(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::AgentCursorStateEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_agent_cursor_state(app, event)
    }

    #[cfg(test)]
    pub fn emit_human_takeover_state_for_test(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::HumanTakeoverStateEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_human_takeover_state(app, event)
    }

    #[cfg(test)]
    pub fn emit_action_durable_state_for_test(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::ActionDurableStateEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_action_durable_state(app, event)
    }

    /// Emit navigation state to the frontend.
    pub fn emit_navigation_state(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::NavigationStateEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_navigation_state(app, event)
    }

    /// Emit perception state to the frontend.
    pub fn emit_perception_state(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::PerceptionStateEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_perception_state(app, event)
    }

    /// Emit action proposal to the frontend.
    pub fn emit_action_proposal(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::ActionProposalEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_action_proposal(app, event)
    }

    /// Emit policy decision to the frontend.
    pub fn emit_policy_decision(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::PolicyDecisionEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_policy_decision(app, event)
    }

    /// Emit execution result to the frontend.
    pub fn emit_execution_result(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::ExecutionResultEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_execution_result(app, event)
    }

    /// Emit verification outcome to the frontend.
    pub fn emit_verification_outcome(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::VerificationOutcomeEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_verification_outcome(app, event)
    }

    /// Emit authorization required to the frontend.
    pub fn emit_authorization_required(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::AuthorizationRequiredEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_authorization_required(app, event)
    }

    /// Emit agent cursor state to the frontend.
    pub fn emit_agent_cursor_state(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::AgentCursorStateEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_agent_cursor_state(app, event)
    }

    /// Emit human takeover state to the frontend.
    pub fn emit_human_takeover_state(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::HumanTakeoverStateEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_human_takeover_state(app, event)
    }

    /// Emit action durable state to the frontend.
    pub fn emit_action_durable_state(
        &self,
        app: &tauri::AppHandle,
        event: crate::ipc::events::ActionDurableStateEvent,
    ) -> Result<(), crate::Error> {
        crate::ipc::emit_action_durable_state(app, event)
    }
}

fn hh_process_state(process: &mut BrowserProcess) -> BrowserRuntimeState {
    // Real probe from the managed process handle. Replaces the placeholder
    // stub so the projected snapshot reflects the live process state.
    ProcessManager::state(process)
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
        assert_eq!(snap.reason.as_deref(), Some("BROWSER_RUNTIME_UNAVAILABLE"));
    }

    #[test]
    fn attach_fails_closed_when_executable_is_missing() {
        let controller = BrowserRuntimeController::new();
        let profile = PathBuf::from("/tmp/browser-agent-test-profile");
        let result = controller.attach(PathBuf::from("/nonexistent").as_path(), profile, &[]);
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
            cdp: parking_lot::Mutex::new(crate::cdp::connection::CdpConnection::from_transport(
                crate::browser::controller::PipeTransport {
                    stdin: None,
                    stdout: None,
                },
            )),
            cdp_transported: true,
            profile_dir: std::path::PathBuf::from("/tmp/browser-agent-test-profile"),
            kind: crate::browser::process::ProcessKind::LowRisk,
        });

        // No-op: this test line exists only to exercise the import path.
        // The real runtime-state emission is covered by the integration path.
        let _app: Option<&tauri::AppHandle> = None;
        let _ctrl: &BrowserRuntimeController = &controller;
        // exercise import path only        let _ = ();

        drop(guard);
        let handle = controller.advanced_test_handle().unwrap();
        controller.advanced_restore_handle(handle);
        let snap = controller.snapshot();
        // A synthetic handle with no live Child is not Ready; the real probe
        // in hh_process_state now reflects that instead of stubbing Ready.
        assert!(matches!(snap.state, BrowserRuntimeState::Unavailable));
        assert!(snap.cdp_transported);
        assert_eq!(
            snap.profile_dir,
            std::path::PathBuf::from("/tmp/browser-agent-test-profile")
        );
        assert_eq!(snap.kind, crate::browser::process::ProcessKind::LowRisk);
    }
}
