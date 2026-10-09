// Browser Agent — local-first browser automation runtime
//
// Architecture (C-01, C-138):
//   5 engines -> orchestrator-owned progress -> semantic state graph
//   -> deterministic baseline -> tiered policy -> typed execution
//   -> independent verification -> immutable skills
//
// Security posture:
//   - Rust owns task authority, policy, authorization tier, verification,
//     epochs, taint/provenance, secrets, egress control, model integrity,
//     audit chain, hard stops, recovery decisions.
//   - Frontend is a projection of typed events. It does not invent
//     security truth (C-17, C-18).
//
// This file is the crate root; subsystem modules live under src/.
// The `runtime-core` feature gates subsystems that depend on the browser
// runtime being attached; app-shell code compiles without it.
// Where a phase is scheduled but not yet delivered, we use explicit
// gated stubs and NOT_IMPLEMENTED_YET markers rather than fake success
// paths (per IDE prompt rule 8).

pub mod app;
pub mod config;
pub mod error;
pub mod ipc;

pub mod audit;
pub mod browser;
pub mod cdp;
pub mod core;
pub mod inference;
pub mod net;
pub mod security;
pub mod storage;

pub use app::diagnostics::*;
pub use app::lifecycle::*;
pub use app::settings::*;
pub use config::RuntimeConfig;
pub use error::Error;

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

fn next_task_id() -> u64 {
    NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed)
}

pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            ipc::commands::app_submit_task,
            ipc::commands::app_abort_task,
            ipc::commands::app_begin_human_handoff,
            ipc::commands::app_confirm_authorization,
            ipc::commands::app_deny_authorization,
        ])
        .setup(|app| {
            #[cfg(any(debug_assertions, feature = "runtime-core"))]
            let _ = app;
            Ok(())
        })
        .run(tauri::generate_context!())
}

pub fn init() {
    tracing::debug!("browser-agent runtime core initialized");
}

#[cfg(test)]
mod tests {
    #[test]
    fn init_is_idempotent_and_does_not_panic() {
        super::init();
        super::init();
    }
}
