//! Canonical user-facing copy tables (BA-STATE-001, BA-READY-001, BA-VERIFY-001).
//!
//! This module is the single source of the status vocabulary. The Rust runtime
//! stamps every task event with the label from these tables, and
//! [`copy_table_json`] exports the identical tables as a checked-in contract
//! file that the first-party UI imports. Implementations must not invent a
//! second, competing copy table.

use serde_json::json;

use crate::errorcopy::ErrorCase;
use crate::task::{ReadinessState, TaskState, VerificationOutcome, WaitReason};

/// Copy shown when two consecutive identical actions make no progress
/// (BA-STUCK-001).
pub const STUCK_LABEL: &str = "The agent appears stuck";

/// Controls offered when the agent appears stuck (BA-STUCK-001).
pub const STUCK_CONTROLS: [&str; 3] = ["Retry observation", "Take over", "Cancel task"];

/// The canonical copy table has no entry for the requested combination.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CopyTableError {
    /// `WAITING_FOR_USER` was requested without a wait reason.
    #[error("WAITING_FOR_USER has no label without a wait reason")]
    MissingWaitReason,
    /// A wait reason was supplied for a state that has no reason-specific copy.
    #[error("state {0} has no reason-specific copy")]
    UnexpectedWaitReason(TaskState),
}

/// Returns the primary user-facing status label for a lifecycle state.
///
/// While the task is `WAITING_FOR_USER` the label comes from the typed wait
/// reason; there is no generic fallback, because a generic “Continue” action is
/// exactly what BA-STATE-002 forbids.
#[must_use]
pub fn task_state_label(state: TaskState, wait_reason: Option<WaitReason>) -> Option<&'static str> {
    if state == TaskState::WaitingForUser {
        return wait_reason.map(wait_reason_label);
    }
    if wait_reason.is_some() {
        return None;
    }
    Some(match state {
        TaskState::Created => "Task received",
        TaskState::Normalizing => "Preparing task",
        TaskState::Observing => "Inspecting page",
        TaskState::Proposing => "Preparing next step",
        TaskState::PolicyCheck => "Checking permission",
        TaskState::Executing => "Performing action",
        TaskState::Verifying => "Checking result",
        TaskState::UserTakeover => "You're in control",
        TaskState::Paused => "Paused",
        TaskState::Blocked => "Blocked by policy",
        TaskState::Failed => "Task failed",
        TaskState::Cancelled => "Task cancelled",
        TaskState::Completed => "Task ended",
        TaskState::WaitingForUser => return None,
    })
}

/// Returns the label for a typed wait reason.
#[must_use]
pub const fn wait_reason_label(reason: WaitReason) -> &'static str {
    match reason {
        WaitReason::ActionApproval => "Waiting for your approval",
        WaitReason::UserInput => "Needs your input",
        WaitReason::Authentication => "Waiting for you to sign in",
        WaitReason::TaskBudget => "Task budget reached",
    }
}

/// Returns the controls named by the specification for a wait reason.
///
/// An empty slice means the specification names no control, so the UI must not
/// invent one.
#[must_use]
pub const fn wait_reason_controls(reason: WaitReason) -> &'static [&'static str] {
    match reason {
        WaitReason::ActionApproval => &["Approve action", "Cancel"],
        WaitReason::UserInput => &[],
        WaitReason::Authentication => &["Take over", "Cancel task"],
        WaitReason::TaskBudget => &[
            "Finish with current results",
            "Extend task budget",
            "Cancel task",
        ],
    }
}

/// Returns the independent verification outcome label (BA-VERIFY-001).
#[must_use]
pub const fn verification_label(outcome: VerificationOutcome) -> &'static str {
    match outcome {
        VerificationOutcome::VerifiedSuccess => "Verified success",
        VerificationOutcome::LikelySuccess => "Likely succeeded \u{2014} not fully verified",
        VerificationOutcome::Unknown => "Result unknown",
        VerificationOutcome::LikelyFailure => "Likely did not succeed",
        VerificationOutcome::VerifiedFailure => "Verified failure",
    }
}

/// Returns the explanatory sentence for a verification outcome.
#[must_use]
pub const fn verification_meaning(outcome: VerificationOutcome) -> &'static str {
    match outcome {
        VerificationOutcome::VerifiedSuccess => {
            "A task-specific postcondition is supported by sufficient independent evidence."
        }
        VerificationOutcome::LikelySuccess => {
            "Positive evidence exists, but a required proof is missing or incomplete."
        }
        VerificationOutcome::Unknown => {
            "The side effect may have happened, or current evidence is insufficient or conflicting."
        }
        VerificationOutcome::LikelyFailure => {
            "Evidence suggests the postcondition was not achieved, but is inconclusive."
        }
        VerificationOutcome::VerifiedFailure => {
            "Independent evidence establishes that the intended postcondition failed."
        }
    }
}

/// Returns the model capability readiness label (BA-READY-001).
#[must_use]
pub const fn readiness_label(state: ReadinessState) -> &'static str {
    match state {
        ReadinessState::NotInstalled => "Model not installed",
        ReadinessState::Installing => "Installing model",
        ReadinessState::IntegrityFailed => "Model failed its integrity check",
        ReadinessState::Incompatible => "Model not compatible with this computer",
        ReadinessState::Loading => "Loading model",
        ReadinessState::Ready => "Model ready",
        ReadinessState::Parked => "Model paused to free memory",
        ReadinessState::RunnerFailed => "Model stopped unexpectedly",
        ReadinessState::ResourceStopped => "Model stopped to protect system performance",
    }
}

/// Returns the controls a readiness state actually supports.
///
/// Controls appear only when their underlying operation exists, so this is a
/// closed list rather than a suggestion.
#[must_use]
pub const fn readiness_controls(state: ReadinessState) -> &'static [&'static str] {
    match state {
        ReadinessState::NotInstalled => &["Install model"],
        ReadinessState::Installing => &["Cancel"],
        ReadinessState::IntegrityFailed => &["Retry verification", "Remove model"],
        ReadinessState::Incompatible
        | ReadinessState::Loading
        | ReadinessState::Ready
        | ReadinessState::ResourceStopped => &[],
        ReadinessState::Parked => &["Resume model"],
        ReadinessState::RunnerFailed => &["Restart model for the next task"],
    }
}

/// Renders the checked-in contract file consumed by the first-party UI.
///
/// Output is deterministic (keys are emitted in sorted order) so the CI
/// contract-drift job can diff it byte for byte.
#[must_use]
pub fn copy_table_json() -> String {
    let status_labels = serde_json::Value::Object(
        TaskState::ALL
            .iter()
            .filter(|state| **state != TaskState::WaitingForUser)
            .map(|state| {
                (
                    state.code().to_owned(),
                    json!(task_state_label(*state, None).unwrap_or_default()),
                )
            })
            .chain(WaitReason::ALL.iter().map(|reason| {
                (
                    format!("WAITING_FOR_USER:{}", reason.code()),
                    json!(wait_reason_label(*reason)),
                )
            }))
            .collect(),
    );

    let task_states = serde_json::Value::Object(
        TaskState::ALL
            .iter()
            .map(|state| {
                let (label, controls) = if *state == TaskState::WaitingForUser {
                    (
                        serde_json::Value::Null,
                        json!(
                            WaitReason::ALL
                                .iter()
                                .map(|reason| wait_reason_controls(*reason))
                                .collect::<Vec<_>>()
                        ),
                    )
                } else {
                    (
                        json!(task_state_label(*state, None).unwrap_or_default()),
                        json!(Vec::<&str>::new()),
                    )
                };
                (
                    state.code().to_owned(),
                    json!({
                        "label": label,
                        "reason_required": *state == TaskState::WaitingForUser,
                        "terminal": state.is_terminal(),
                        "active_work": state.is_active_work(),
                        "next": state.allowed_next().iter().map(|s| s.code()).collect::<Vec<_>>(),
                        "reason_controls": controls,
                    }),
                )
            })
            .collect(),
    );

    let wait_reasons = serde_json::Value::Object(
        WaitReason::ALL
            .iter()
            .map(|reason| {
                (
                    reason.code().to_owned(),
                    json!({
                        "label": wait_reason_label(*reason),
                        "state": TaskState::WaitingForUser.code(),
                        "controls": wait_reason_controls(*reason),
                    }),
                )
            })
            .collect(),
    );

    let verification_outcomes = serde_json::Value::Object(
        VerificationOutcome::ALL
            .iter()
            .map(|outcome| {
                (
                    outcome.code().to_owned(),
                    json!({
                        "label": verification_label(*outcome),
                        "meaning": verification_meaning(*outcome),
                        "verified_success": outcome.is_verified_success(),
                    }),
                )
            })
            .collect(),
    );

    let readiness_states = serde_json::Value::Object(
        ReadinessState::ALL
            .iter()
            .map(|state| {
                (
                    state.code().to_owned(),
                    json!({
                        "label": readiness_label(*state),
                        "controls": readiness_controls(*state),
                        "accepts_tasks": state.accepts_tasks(),
                    }),
                )
            })
            .collect(),
    );

    let document = json!({
        "schema_version": crate::SCHEMA_VERSION,
        "status_labels": status_labels,
        "task_states": task_states,
        "wait_reasons": wait_reasons,
        "verification_outcomes": verification_outcomes,
        "readiness_states": readiness_states,
        "error_copy": serde_json::Value::Object(
            ErrorCase::ALL
                .iter()
                .map(|case| {
                    let copy = case.copy();
                    (
                        case_name(*case).to_owned(),
                        json!({
                            "error_id": copy.error_id,
                            "attempted": copy.attempted,
                            "observed": copy.observed,
                            "known": copy.known,
                            "next_action": copy.next_action,
                        }),
                    )
                })
                .collect(),
        ),
        "stuck": {
            "label": STUCK_LABEL,
            "controls": STUCK_CONTROLS,
        },
    });

    let mut rendered = serde_json::to_string_pretty(&document).unwrap_or_else(|_| "{}".to_owned());
    rendered.push('\n');
    rendered
}

/// Returns the stable JSON key for an error case.
#[must_use]
pub const fn case_name(case: ErrorCase) -> &'static str {
    match case {
        ErrorCase::NavigationFailed => "NAVIGATION_FAILED",
        ErrorCase::ModelNotReady => "MODEL_NOT_READY",
        ErrorCase::RunnerCrashed => "RUNNER_CRASHED",
        ErrorCase::PolicyBlocked => "POLICY_BLOCKED",
        ErrorCase::ApprovalExpired => "APPROVAL_EXPIRED",
        ErrorCase::VerificationUnknown => "VERIFICATION_UNKNOWN",
        ErrorCase::DownloadRejected => "DOWNLOAD_REJECTED",
        ErrorCase::AuditWriteFailed => "AUDIT_WRITE_FAILED",
        ErrorCase::ResourceStop => "RESOURCE_STOP",
        ErrorCase::BrowserHostUnavailable => "BROWSER_HOST_UNAVAILABLE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_state_has_a_label_and_waiting_needs_a_reason() {
        for state in TaskState::ALL {
            if state == TaskState::WaitingForUser {
                assert!(task_state_label(state, None).is_none());
                for reason in WaitReason::ALL {
                    assert!(task_state_label(state, Some(reason)).is_some());
                }
            } else {
                assert!(
                    task_state_label(state, None).is_some(),
                    "{state} has no label"
                );
                assert!(task_state_label(state, Some(WaitReason::UserInput)).is_none());
            }
        }
    }

    #[test]
    fn labels_are_distinct() {
        let mut seen = std::collections::BTreeSet::new();
        for state in TaskState::ALL {
            if state == TaskState::WaitingForUser {
                continue;
            }
            assert!(seen.insert(task_state_label(state, None).unwrap()));
        }
        for reason in WaitReason::ALL {
            assert!(seen.insert(wait_reason_label(reason)));
        }
    }

    #[test]
    fn only_verified_success_is_marked_verified() {
        for outcome in VerificationOutcome::ALL {
            assert_eq!(
                outcome.is_verified_success(),
                outcome == VerificationOutcome::VerifiedSuccess
            );
        }
    }

    #[test]
    fn contract_json_is_stable_and_complete() {
        let rendered = copy_table_json();
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(parsed["schema_version"], json!(crate::SCHEMA_VERSION));
        for state in TaskState::ALL {
            if state == TaskState::WaitingForUser {
                continue;
            }
            let key = state.code();
            assert_eq!(
                parsed["status_labels"][key],
                json!(task_state_label(state, None).unwrap())
            );
        }
        for reason in WaitReason::ALL {
            let key = format!("WAITING_FOR_USER:{}", reason.code());
            assert_eq!(
                parsed["status_labels"][key],
                json!(wait_reason_label(reason))
            );
            assert_eq!(
                parsed["wait_reasons"][reason.code()]["label"],
                json!(wait_reason_label(reason))
            );
        }
        for state in ReadinessState::ALL {
            assert!(
                parsed["readiness_states"][state.code()]["label"] == json!(readiness_label(state))
            );
            for case in ErrorCase::ALL {
                let entry = &parsed["error_copy"][case_name(case)];
                assert_eq!(entry["error_id"], json!(case.copy().error_id));
                assert_eq!(entry["attempted"], json!(case.copy().attempted));
                assert_eq!(entry["next_action"], json!(case.copy().next_action));
            }
            assert_eq!(parsed["stuck"]["label"], json!(STUCK_LABEL));
        }
        assert_eq!(
            copy_table_json(),
            rendered,
            "rendering must be deterministic"
        );
        assert!(rendered.ends_with('\n'));
    }

    #[test]
    fn task_budget_controls_match_the_specification() {
        assert_eq!(
            wait_reason_controls(WaitReason::TaskBudget),
            &[
                "Finish with current results",
                "Extend task budget",
                "Cancel task"
            ]
        );
        assert!(wait_reason_controls(WaitReason::UserInput).is_empty());
    }
}
