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
pub use config::{EgressMode, RuntimeConfig};
pub use error::Error;

use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

fn next_task_id() -> u64 {
    NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed)
}

/// Run the Tauri application to completion.
///
/// Window creation and app setup run on the first event-loop tick
/// (macOS: inside `did_finish_launching`). The `stage = "event_loop"` log
/// is the last stage marker before that tick; any startup failure after
/// it is captured by the panic diagnostics hook installed in `main.rs`.
pub fn run() -> tauri::Result<()> {
    tracing::info!(stage = "builder", "constructing tauri builder");
    let store = std::sync::Arc::new(
        crate::storage::sqlite::SqliteStore::open_in_memory()
            .expect("in-memory sqlite store initialization"),
    );
    let vault = std::sync::Arc::new(crate::core::vault::service::VaultService::new(
        store.clone(),
    ));

    let builder = tauri::Builder::default()
        .manage(std::sync::Arc::new(
            crate::browser::controller::BrowserRuntimeController::new(),
        ))
        .manage(std::sync::Arc::new(
            crate::core::orchestrator::runtime::TaskRegistry::default(),
        ))
        .manage(store)
        .manage(vault)
        .manage(RuntimeConfig {
            model_manifest_path: "models/manifests/phase1-provisional.json".into(),
            installed_models_dir: "models".into(),
            audit_segment_dir: String::new(),
            quarantine_dir: String::new(),
            profiles_dir: String::new(),
            egress_mode: EgressMode::Enforced,
            chromium_executable: None,
            max_steps: None,
            max_llm_calls: None,
            max_recovery_attempts: None,
            max_task_duration_ms: None,
            max_confirmations: None,
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::app_submit_task,
            ipc::commands::app_abort_task,
            ipc::commands::app_list_tasks,
            ipc::commands::app_begin_human_handoff,
            ipc::commands::app_end_human_handoff,
            ipc::commands::app_confirm_authorization,
            ipc::commands::app_deny_authorization,
            ipc::commands::app_model_status,
            ipc::commands::app_egress_status,
            ipc::commands::app_navigate,
            ipc::commands::app_back,
            ipc::commands::app_forward,
            ipc::commands::app_reload,
            ipc::commands::app_vault_status,
            ipc::commands::app_vault_create,
            ipc::commands::app_vault_unlock,
            ipc::commands::app_vault_lock,
            ipc::commands::app_vault_change_password,
            ipc::commands::app_vault_list_items,
            ipc::commands::app_vault_add_item,
            ipc::commands::app_vault_delete_item,
            ipc::commands::app_audit_verify_chain,
            ipc::commands::app_audit_records,
            ipc::commands::app_skills_list,
            ipc::commands::app_skill_approve_version,
            ipc::commands::app_quarantine_list,
            ipc::commands::app_quarantine_verify,
            ipc::commands::app_profiles_list,
            ipc::commands::app_profiles_purge,
            ipc::commands::app_get_settings,
            ipc::commands::app_update_settings,
        ]);
    tracing::info!(stage = "event_loop", "entering tauri event loop");
    builder.run(tauri::generate_context!())
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
