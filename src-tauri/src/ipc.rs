pub mod allowlist;
pub mod commands;
pub mod errors;
pub mod events;

use events::{
    ActionDurableStateEvent, ActionProposalEvent, AgentCursorStateEvent, AppEvent,
    AuthorizationRequiredEvent, AuthorizationResolvedEvent, BrowserRuntimeStateEvent,
    ExecutionResultEvent, HumanTakeoverStateEvent, NavigationStateEvent, PerceptionStateEvent,
    PolicyDecisionEvent, TaskStateEvent, TaskStatus, VerificationOutcomeEvent,
};
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

pub fn emit_browser_runtime_state(
    app: &AppHandle,
    event: BrowserRuntimeStateEvent,
) -> Result<(), crate::Error> {
    app.emit("app-event", AppEvent::BrowserRuntimeStateChanged(event))
        .map_err(|error| {
            crate::Error::Internal(format!("failed to emit browser runtime state: {error}"))
        })
}

pub fn emit_navigation_state(
    app: &AppHandle,
    event: NavigationStateEvent,
) -> Result<(), crate::Error> {
    app.emit("app-event", AppEvent::NavigationStateChanged(event))
        .map_err(|error| {
            crate::Error::Internal(format!("failed to emit navigation state: {error}"))
        })
}

pub fn emit_perception_state(
    app: &AppHandle,
    event: PerceptionStateEvent,
) -> Result<(), crate::Error> {
    app.emit("app-event", AppEvent::PerceptionStateChanged(event))
        .map_err(|error| {
            crate::Error::Internal(format!("failed to emit perception state: {error}"))
        })
}

pub fn emit_action_proposal(
    app: &AppHandle,
    event: ActionProposalEvent,
) -> Result<(), crate::Error> {
    app.emit("app-event", AppEvent::ActionProposed(event))
        .map_err(|error| crate::Error::Internal(format!("failed to emit action proposal: {error}")))
}

pub fn emit_policy_decision(
    app: &AppHandle,
    event: PolicyDecisionEvent,
) -> Result<(), crate::Error> {
    app.emit("app-event", AppEvent::PolicyDecision(event))
        .map_err(|error| crate::Error::Internal(format!("failed to emit policy decision: {error}")))
}

pub fn emit_execution_result(
    app: &AppHandle,
    event: ExecutionResultEvent,
) -> Result<(), crate::Error> {
    app.emit("app-event", AppEvent::ExecutionResult(event))
        .map_err(|error| {
            crate::Error::Internal(format!("failed to emit execution result: {error}"))
        })
}

pub fn emit_verification_outcome(
    app: &AppHandle,
    event: VerificationOutcomeEvent,
) -> Result<(), crate::Error> {
    app.emit("app-event", AppEvent::VerificationOutcome(event))
        .map_err(|error| {
            crate::Error::Internal(format!("failed to emit verification outcome: {error}"))
        })
}

pub fn emit_authorization_required(
    app: &AppHandle,
    event: AuthorizationRequiredEvent,
) -> Result<(), crate::Error> {
    app.emit("app-event", AppEvent::AuthorizationRequired(event))
        .map_err(|error| {
            crate::Error::Internal(format!("failed to emit authorization required: {error}"))
        })
}

pub fn emit_authorization_resolved(
    app: &AppHandle,
    event: AuthorizationResolvedEvent,
) -> Result<(), crate::Error> {
    app.emit("app-event", AppEvent::AuthorizationResolved(event))
        .map_err(|error| {
            crate::Error::Internal(format!("failed to emit authorization resolved: {error}"))
        })
}

pub fn emit_agent_cursor_state(
    app: &AppHandle,
    event: AgentCursorStateEvent,
) -> Result<(), crate::Error> {
    app.emit("app-event", AppEvent::AgentCursorStateChanged(event))
        .map_err(|error| {
            crate::Error::Internal(format!("failed to emit agent cursor state: {error}"))
        })
}

pub fn emit_human_takeover_state(
    app: &AppHandle,
    event: HumanTakeoverStateEvent,
) -> Result<(), crate::Error> {
    app.emit("app-event", AppEvent::HumanTakeoverStateChanged(event))
        .map_err(|error| {
            crate::Error::Internal(format!("failed to emit human takeover state: {error}"))
        })
}

pub fn emit_action_durable_state(
    app: &AppHandle,
    event: ActionDurableStateEvent,
) -> Result<(), crate::Error> {
    app.emit("app-event", AppEvent::ActionDurableStateChanged(event))
        .map_err(|error| {
            crate::Error::Internal(format!("failed to emit action durable state: {error}"))
        })
}
