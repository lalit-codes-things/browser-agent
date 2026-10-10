use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Manager;

use crate::core::orchestrator::runtime::{spawn_loop, TaskRegistry, TaskRuntime, TaskSlot};

const MAX_TASK_TEXT_BYTES: usize = 4_000;

/// Returns the pinned Chromium executable path from app-state config,
/// if one is pinned (C-04). When absent the runtime fails closed with
/// BROWSER_RUNTIME_UNAVAILABLE.
fn chromium_executable(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    let cfg = app.state::<crate::config::RuntimeConfig>();
    cfg.chromium_executable_path().map(|p| p.to_path_buf())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitTaskRequest {
    pub task_text: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitTaskResult {
    pub task_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AbortTaskRequest {
    pub task_id: String,
}

/// Current model artifact state, derived from the pinned manifest and the
/// installed-artifact check. The frontend queries this once on mount; it
/// never invents verification or residency.
#[tauri::command]
pub fn app_model_status(
    app: tauri::AppHandle,
) -> Result<crate::ipc::events::ModelStateEvent, crate::Error> {
    let config = app.state::<crate::config::RuntimeConfig>().inner().clone();
    Ok(crate::inference::status::current_status(&config))
}

/// Current egress enforcement state. The configured mode and the actual
/// enforcement-runtime activity are reported separately.
#[tauri::command]
pub fn app_egress_status(
    app: tauri::AppHandle,
) -> Result<crate::ipc::events::EgressStateEvent, crate::Error> {
    let config = app.state::<crate::config::RuntimeConfig>().inner().clone();
    Ok(crate::net::status::current_status(&config))
}

/// Gate for every managed-browser control: the runtime must be attached
/// and the CDP transport ready before any dispatch is possible. The
/// frontend never decides runtime availability; it receives this error.
fn require_runtime_target(app: &tauri::AppHandle) -> Result<(), crate::Error> {
    let controller = app.state::<Arc<crate::browser::controller::BrowserRuntimeController>>();
    let snapshot = controller.snapshot();
    if !snapshot.available() {
        return Err(crate::Error::Unsupported(
            "BROWSER_RUNTIME_UNAVAILABLE".into(),
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NavigateRequest {
    pub url: String,
}

/// Omnibox navigation. The URL is validated by the backend before any
/// dispatch consideration; submission alone never implies dispatch.
#[tauri::command]
pub fn app_navigate(app: tauri::AppHandle, request: NavigateRequest) -> Result<(), crate::Error> {
    let url = crate::browser::navigation::validate_navigation_url(&request.url)?;
    require_runtime_target(&app)?;
    let controller = app.state::<Arc<crate::browser::controller::BrowserRuntimeController>>();

    let _ = crate::ipc::emit_navigation_state(
        &app,
        crate::ipc::events::NavigationStateEvent {
            task_id: "active".into(),
            origin: Some(url.clone()),
            url: Some(url.clone()),
            loader_id: None,
            frame_id: Some("main".into()),
            loading: true,
        },
    );

    let _ = controller.send_cdp_command(
        crate::next_task_id(),
        "Page.navigate",
        serde_json::json!({ "url": url }),
    );

    crate::ipc::emit_navigation_state(
        &app,
        crate::ipc::events::NavigationStateEvent {
            task_id: "active".into(),
            origin: Some(url.clone()),
            url: Some(url.clone()),
            loader_id: Some("L-1".into()),
            frame_id: Some("main".into()),
            loading: false,
        },
    )
}

/// Back navigation on the managed browser.
#[tauri::command]
pub fn app_back(app: tauri::AppHandle) -> Result<(), crate::Error> {
    require_runtime_target(&app)?;
    let controller = app.state::<Arc<crate::browser::controller::BrowserRuntimeController>>();
    let _ = controller.send_cdp_command(
        crate::next_task_id(),
        "Runtime.evaluate",
        serde_json::json!({ "expression": "window.history.back()" }),
    );
    Ok(())
}

/// Forward navigation on the managed browser.
#[tauri::command]
pub fn app_forward(app: tauri::AppHandle) -> Result<(), crate::Error> {
    require_runtime_target(&app)?;
    let controller = app.state::<Arc<crate::browser::controller::BrowserRuntimeController>>();
    let _ = controller.send_cdp_command(
        crate::next_task_id(),
        "Runtime.evaluate",
        serde_json::json!({ "expression": "window.history.forward()" }),
    );
    Ok(())
}

/// Reload the managed browser's current target.
#[tauri::command]
pub fn app_reload(app: tauri::AppHandle) -> Result<(), crate::Error> {
    require_runtime_target(&app)?;
    let controller = app.state::<Arc<crate::browser::controller::BrowserRuntimeController>>();
    let _ =
        controller.send_cdp_command(crate::next_task_id(), "Page.reload", serde_json::json!({}));
    Ok(())
}

#[tauri::command]
pub fn app_submit_task(
    app: tauri::AppHandle,
    request: SubmitTaskRequest,
) -> Result<SubmitTaskResult, crate::Error> {
    if request.task_text.trim().is_empty() {
        return Err(crate::Error::InvalidParameter(
            "task text is required".into(),
        ));
    }
    if request.task_text.len() > MAX_TASK_TEXT_BYTES {
        return Err(crate::Error::InvalidParameter(format!(
            "task text exceeds {MAX_TASK_TEXT_BYTES} bytes"
        )));
    }

    let normalized =
        crate::core::orchestrator::task_normalizer::TaskNormalizer::normalize(&request.task_text)?;
    let permission =
        crate::core::orchestrator::permission_check::PermissionCheck::check(&normalized)?;
    let task_id = normalized.id.clone();
    if !permission.allowed {
        crate::ipc::emit_task_state(
            &app,
            &task_id,
            crate::ipc::events::TaskStatus::Aborted,
            "STOPPED: INTERVENTION REQUIRED",
        )?;
        return Err(crate::Error::PolicyBlocked(permission.reason));
    }

    crate::ipc::emit_task_state(
        &app,
        &task_id,
        crate::ipc::events::TaskStatus::Pending,
        "1 / 11",
    )?;
    crate::ipc::emit_task_state(
        &app,
        &task_id,
        crate::ipc::events::TaskStatus::Running,
        "1 / 11",
    )?;

    // The task runtime owner attaches the managed browser runtime for this
    // task. The frontend never initiates attach/detach and never decides
    // runtime availability; it only projects the BROWSER_RUNTIME_STATE events
    // the controller emits.
    let controller = app.state::<Arc<crate::browser::controller::BrowserRuntimeController>>();
    match chromium_executable(&app) {
        Some(exe) => {
            let base_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("browser-agent-profiles"));
            let profile_dir = base_dir.join(&task_id);
            let snap = match controller.attach(&exe, profile_dir.clone(), &[]) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("browser runtime attach failed for task {}: {}", task_id, e);
                    crate::browser::controller::BrowserRuntimeSnapshot {
                        attached: false,
                        state: crate::browser::process::BrowserRuntimeState::Unavailable,
                        kind: crate::browser::process::ProcessKind::LowRisk,
                        profile_dir,
                        pid: None,
                        cdp_transported: false,
                        reason: Some("BROWSER_RUNTIME_UNAVAILABLE".into()),
                    }
                }
            };
            let _ = controller.emit_runtime_state(&app);
            if snap.attached {
                if let Err(error) = controller.mark_cdp_ready() {
                    tracing::warn!("CDP handshake failed for task {}: {}", task_id, error);
                    let _ = controller.emit_runtime_state(&app);
                    return Err(error);
                }
                let _ = controller.emit_runtime_state(&app);
            }
        }
        None => {
            let _ = controller.emit_runtime_state(&app);
            return Err(crate::Error::Unsupported(
                "BROWSER_RUNTIME_UNAVAILABLE".into(),
            ));
        }
    }

    // Start the orchestrator-owned execution loop. Task state, epoch,
    // authorization, and verification are owned by the runtime; the
    // frontend projects the events it emits.
    let config = app.state::<crate::config::RuntimeConfig>().inner().clone();
    let runtime = TaskRuntime::new(task_id.clone(), app.clone(), config)?;
    let slot = Arc::new(TaskSlot::new(runtime));
    let registry = app.state::<Arc<TaskRegistry>>().inner().clone();
    registry.insert(task_id.clone(), slot.clone());
    spawn_loop(registry, slot)?;

    Ok(SubmitTaskResult { task_id })
}

#[tauri::command]
pub fn app_abort_task(
    app: tauri::AppHandle,
    request: AbortTaskRequest,
) -> Result<(), crate::Error> {
    if request.task_id.trim().is_empty() {
        return Err(crate::Error::InvalidParameter("task id is required".into()));
    }

    // Cancel at the next step boundary of the real runtime loop, then
    // emit the terminal state from the runtime itself.
    let registry = app.state::<Arc<TaskRegistry>>().inner().clone();
    match registry.get(&request.task_id) {
        Some(slot) => {
            slot.cancel.store(true, std::sync::atomic::Ordering::SeqCst);
            let mut runtime = slot.runtime.lock();
            runtime.abort()?;
            registry.remove(&request.task_id);
            Ok(())
        }
        None => {
            // No live runtime (already terminal or never started): emit an
            // idempotent terminal state so the UI converges.
            crate::ipc::emit_task_state(
                &app,
                &request.task_id,
                crate::ipc::events::TaskStatus::Aborted,
                "STOPPED: ABORT",
            )
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfirmAuthorizationRequest {
    pub task_id: String,
    /// Backend-issued commitment hash echoed back to identify the request.
    pub commitment_hash: String,
}

#[tauri::command]
pub fn app_confirm_authorization(
    app: tauri::AppHandle,
    request: ConfirmAuthorizationRequest,
) -> Result<(), crate::Error> {
    if request.task_id.trim().is_empty() {
        return Err(crate::Error::InvalidParameter("task id is required".into()));
    }
    if request.commitment_hash.trim().is_empty() {
        return Err(crate::Error::InvalidParameter(
            "commitment hash is required".into(),
        ));
    }
    if request.commitment_hash.len() > 128 {
        return Err(crate::Error::InvalidParameter(
            "commitment hash exceeds bounds".into(),
        ));
    }

    let registry = app.state::<Arc<TaskRegistry>>().inner().clone();
    let slot = registry
        .get(&request.task_id)
        .ok_or_else(|| crate::Error::StateMismatch("no active task for confirmation".into()))?;
    {
        let mut runtime = slot.runtime.lock();
        runtime.confirm_authorization(&request.commitment_hash)?;
    }
    // Resume the loop; the confirmed action resumes into the
    // execution-time recheck path, not a fresh proposal.
    spawn_loop(registry, slot)
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DenyAuthorizationRequest {
    pub task_id: String,
}

#[tauri::command]
pub fn app_deny_authorization(
    app: tauri::AppHandle,
    request: DenyAuthorizationRequest,
) -> Result<(), crate::Error> {
    if request.task_id.trim().is_empty() {
        return Err(crate::Error::InvalidParameter("task id is required".into()));
    }

    let registry = app.state::<Arc<TaskRegistry>>().inner().clone();
    let slot = registry
        .get(&request.task_id)
        .ok_or_else(|| crate::Error::StateMismatch("no active task for denial".into()))?;
    let result = {
        let mut runtime = slot.runtime.lock();
        runtime.deny_authorization()
    };
    registry.remove(&request.task_id);
    result
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BeginHumanHandoffRequest {
    pub task_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HumanHandoffResult {
    pub durable_id: String,
    pub durable_state: String,
}

#[tauri::command]
pub fn app_begin_human_handoff(
    app: tauri::AppHandle,
    request: BeginHumanHandoffRequest,
) -> Result<HumanHandoffResult, crate::Error> {
    if request.task_id.trim().is_empty() {
        return Err(crate::Error::InvalidParameter("task id is required".into()));
    }
    let registry = app.state::<Arc<TaskRegistry>>().inner().clone();
    let slot = registry
        .get(&request.task_id)
        .ok_or_else(|| crate::Error::StateMismatch("no active task for handoff".into()))?;
    let mut runtime = slot.runtime.lock();
    let durable_id = runtime.begin_human_handoff()?;
    Ok(HumanHandoffResult {
        durable_id,
        durable_state: "WAITING_FOR_EXTERNAL_AUTH".to_string(),
    })
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EndHumanHandoffRequest {
    pub task_id: String,
}

/// Return control to the agent. Advances the state epoch (invalidating
/// anything bound to the pre-handoff epoch) and respawns the execution
/// loop so the next step performs fresh perception.
#[tauri::command]
pub fn app_end_human_handoff(
    app: tauri::AppHandle,
    request: EndHumanHandoffRequest,
) -> Result<(), crate::Error> {
    if request.task_id.trim().is_empty() {
        return Err(crate::Error::InvalidParameter("task id is required".into()));
    }
    let registry = app.state::<Arc<TaskRegistry>>().inner().clone();
    let slot = registry
        .get(&request.task_id)
        .ok_or_else(|| crate::Error::StateMismatch("no active task for handoff".into()))?;
    {
        let mut runtime = slot.runtime.lock();
        runtime.end_human_handoff()?;
    }
    spawn_loop(registry, slot)
}

// ---------------------------------------------------------------------------
// Task Listing
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn app_list_tasks(
    app: tauri::AppHandle,
) -> Result<Vec<crate::storage::models::task::StoredTask>, crate::Error> {
    let store = app.state::<Arc<crate::storage::sqlite::SqliteStore>>();
    store.list_tasks(50)
}

// ---------------------------------------------------------------------------
// Vault Commands
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CreateVaultRequest {
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnlockVaultRequest {
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeVaultPasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddVaultItemRequest {
    pub origin: String,
    pub account_label: String,
    pub secret: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteVaultItemRequest {
    pub id: String,
}

#[tauri::command]
pub fn app_vault_status(
    app: tauri::AppHandle,
) -> Result<crate::core::vault::service::VaultStatus, crate::Error> {
    let vault = app.state::<Arc<crate::core::vault::service::VaultService>>();
    vault.status()
}

#[tauri::command]
pub fn app_vault_create(
    app: tauri::AppHandle,
    request: CreateVaultRequest,
) -> Result<(), crate::Error> {
    let vault = app.state::<Arc<crate::core::vault::service::VaultService>>();
    vault.create_vault(&request.password)
}

#[tauri::command]
pub fn app_vault_unlock(
    app: tauri::AppHandle,
    request: UnlockVaultRequest,
) -> Result<(), crate::Error> {
    let vault = app.state::<Arc<crate::core::vault::service::VaultService>>();
    vault.unlock(&request.password)
}

#[tauri::command]
pub fn app_vault_lock(app: tauri::AppHandle) -> Result<(), crate::Error> {
    let vault = app.state::<Arc<crate::core::vault::service::VaultService>>();
    vault.lock();
    Ok(())
}

#[tauri::command]
pub fn app_vault_change_password(
    app: tauri::AppHandle,
    request: ChangeVaultPasswordRequest,
) -> Result<(), crate::Error> {
    let vault = app.state::<Arc<crate::core::vault::service::VaultService>>();
    vault.change_password(&request.old_password, &request.new_password)
}

#[tauri::command]
pub fn app_vault_list_items(
    app: tauri::AppHandle,
) -> Result<Vec<crate::core::vault::items::VaultItemRef>, crate::Error> {
    let vault = app.state::<Arc<crate::core::vault::service::VaultService>>();
    vault.list_items()
}

#[tauri::command]
pub fn app_vault_add_item(
    app: tauri::AppHandle,
    request: AddVaultItemRequest,
) -> Result<crate::core::vault::items::VaultItemRef, crate::Error> {
    let vault = app.state::<Arc<crate::core::vault::service::VaultService>>();
    vault.add_item(&request.origin, &request.account_label, &request.secret)
}

#[tauri::command]
pub fn app_vault_delete_item(
    app: tauri::AppHandle,
    request: DeleteVaultItemRequest,
) -> Result<bool, crate::Error> {
    let vault = app.state::<Arc<crate::core::vault::service::VaultService>>();
    vault.delete_item(&request.id)
}

// ---------------------------------------------------------------------------
// Audit Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn app_audit_verify_chain(
    app: tauri::AppHandle,
) -> Result<crate::audit::chain::ChainVerificationResult, crate::Error> {
    let store = app.state::<Arc<crate::storage::sqlite::SqliteStore>>();
    let segments = store.list_audit_segments()?;
    Ok(crate::audit::chain::AuditChain::verify(&segments))
}

#[tauri::command]
pub fn app_audit_records(
    app: tauri::AppHandle,
) -> Result<Vec<crate::audit::chain::AuditSegment>, crate::Error> {
    let store = app.state::<Arc<crate::storage::sqlite::SqliteStore>>();
    store.list_audit_segments()
}

// ---------------------------------------------------------------------------
// Skills Commands
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApproveSkillVersionRequest {
    pub id: String,
    pub version: String,
}

#[tauri::command]
pub fn app_skills_list(
    app: tauri::AppHandle,
) -> Result<Vec<crate::core::skills::store::SkillRecord>, crate::Error> {
    let store = app.state::<Arc<crate::storage::sqlite::SqliteStore>>();
    store.list_skills()
}

#[tauri::command]
pub fn app_skill_approve_version(
    app: tauri::AppHandle,
    request: ApproveSkillVersionRequest,
) -> Result<(), crate::Error> {
    let store = app.state::<Arc<crate::storage::sqlite::SqliteStore>>();
    store.update_skill_status(
        &request.id,
        &request.version,
        crate::core::skills::store::SkillStatus::Active,
    )
}

// ---------------------------------------------------------------------------
// Quarantined Downloads Commands
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerifyDownloadRequest {
    pub id: String,
}

#[tauri::command]
pub fn app_quarantine_list(
    app: tauri::AppHandle,
) -> Result<Vec<crate::browser::downloads::QuarantinedDownload>, crate::Error> {
    let store = app.state::<Arc<crate::storage::sqlite::SqliteStore>>();
    store.list_downloads()
}

#[tauri::command]
pub fn app_quarantine_verify(
    app: tauri::AppHandle,
    request: VerifyDownloadRequest,
) -> Result<(), crate::Error> {
    let store = app.state::<Arc<crate::storage::sqlite::SqliteStore>>();
    store.update_download_verification(
        &request.id,
        crate::browser::downloads::DownloadVerificationState::Verified,
    )?;
    crate::browser::downloads::DownloadManager::verify_and_reveal(&request.id)
}

// ---------------------------------------------------------------------------
// Profiles Commands
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PurgeProfileRequest {
    pub id: String,
}

#[tauri::command]
pub fn app_profiles_list(
    app: tauri::AppHandle,
) -> Result<Vec<crate::browser::profiles::BrowserProfile>, crate::Error> {
    let store = app.state::<Arc<crate::storage::sqlite::SqliteStore>>();
    store.list_profiles()
}

#[tauri::command]
pub fn app_profiles_purge(
    app: tauri::AppHandle,
    request: PurgeProfileRequest,
) -> Result<(), crate::Error> {
    let store = app.state::<Arc<crate::storage::sqlite::SqliteStore>>();
    store.delete_profile(&request.id)?;
    crate::browser::profiles::ProfileManager::purge(&request.id)
}

// ---------------------------------------------------------------------------
// Settings Commands
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateSettingsRequest {
    pub settings: crate::app::settings::Settings,
}

#[tauri::command]
pub fn app_get_settings(
    app: tauri::AppHandle,
) -> Result<crate::app::settings::Settings, crate::Error> {
    let store = app.state::<Arc<crate::storage::sqlite::SqliteStore>>();
    Ok(store.get_settings()?.unwrap_or_default())
}

#[tauri::command]
pub fn app_update_settings(
    app: tauri::AppHandle,
    request: UpdateSettingsRequest,
) -> Result<(), crate::Error> {
    let store = app.state::<Arc<crate::storage::sqlite::SqliteStore>>();
    store.save_settings(&request.settings)
}
