use serde::{Deserialize, Serialize};

const MAX_TASK_TEXT_BYTES: usize = 4_000;

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
        "1 / 1",
    )?;
    crate::ipc::emit_task_state(
        &app,
        &task_id,
        crate::ipc::events::TaskStatus::Running,
        "1 / 1",
    )?;
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
    crate::ipc::emit_task_state(
        &app,
        &request.task_id,
        crate::ipc::events::TaskStatus::Aborted,
        "1 / 1",
    )
}

#[tauri::command]
pub fn app_confirm_authorization() -> Result<(), crate::Error> {
    Err(crate::Error::NotImplemented(
        "authorization approval requires a backend-issued commitment".into(),
    ))
}

#[tauri::command]
pub fn app_deny_authorization() -> Result<(), crate::Error> {
    Err(crate::Error::PolicyBlocked(
        "authorization denied; task must stop".into(),
    ))
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
    _request: BeginHumanHandoffRequest,
) -> Result<HumanHandoffResult, crate::Error> {
    Err(crate::Error::PolicyBlocked(
        "automated payment submission is disabled in this product; hand control to the user".into(),
    ))
}
