// Task State Machine.
//
// Deterministic transitions for task lifecycle. No frontend-owned
// security transitions.
//
// Possible states: PENDING, RUNNING, VERIFIED, LIKELY_SUCCESS, UNKNOWN,
// FAILED, ABORTED (per DESIGN.md section 14.1).

use crate::core::orchestrator::progress::TaskState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionError {
    InvalidFromState,
    InvalidToState,
    NotAllowed,
}

pub struct StateMachine;

impl StateMachine {
    pub fn can_transition(from: TaskState, to: TaskState) -> bool {
        Self::allowed(from, &to)
    }

    fn allowed(from: TaskState, to: &TaskState) -> bool {
        matches!(
            (from, to),
            (TaskState::Pending, TaskState::Running)
                | (TaskState::Running, TaskState::Verified)
                | (TaskState::Running, TaskState::LikelySuccess)
                | (TaskState::Running, TaskState::Unknown)
                | (TaskState::Running, TaskState::Failed)
                | (TaskState::Running, TaskState::Aborted)
                | (TaskState::LikelySuccess, TaskState::Verified)
                | (TaskState::Unknown, TaskState::Running)
                | (TaskState::Unknown, TaskState::Failed)
                | (TaskState::Unknown, TaskState::Aborted)
                | (TaskState::Failed, TaskState::Aborted)
                | (TaskState::Aborted, TaskState::Aborted)
        )
    }

    pub fn transition(from: TaskState, to: TaskState) -> Result<TaskState, TransitionError> {
        if Self::allowed(from, &to) {
            Ok(to)
        } else {
            Err(TransitionError::NotAllowed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::orchestrator::progress::TaskState::*;

    #[test]
    fn valid_running_to_verified() {
        assert!(StateMachine::can_transition(Running, Verified));
    }

    #[test]
    fn invalid_pending_to_verified() {
        assert!(!StateMachine::can_transition(Pending, Verified));
    }

    #[test]
    fn aborted_is_stable() {
        assert!(StateMachine::can_transition(Aborted, Aborted));
    }

    #[test]
    fn transition_returns_state() {
        assert_eq!(StateMachine::transition(Running, Verified), Ok(Verified));
        assert!(StateMachine::transition(Pending, Verified).is_err());
    }
}
