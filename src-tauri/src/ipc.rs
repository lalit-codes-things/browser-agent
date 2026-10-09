pub mod allowlist;
pub mod commands;
pub mod errors;
pub mod events;

use events::{AppEvent, TaskStateEvent, TaskStatus};
use tauri::{AppHandle, Emitter};

pub fn emit_task_state(
    app: &AppHandle,
    task_id: &str,
    status: TaskStatus,
    step_label: &str,
) -> Result<(), crate::Error> {
    app.emit(
        "app-event",
        AppEvent::TaskStateChanged(TaskStateEvent {
            task_id: task_id.to_owned(),
            status,
            step_label: step_label.to_owned(),
        }),
    )
    .map_err(|error| crate::Error::Internal(format!("failed to emit task state: {error}")))
}
