//! Task lifecycle, wait reasons, action classes and the verification vocabulary
//! (Sections 12.2, 13.2, 15.5, 2.9.1).

use core::fmt;

use serde::{Deserialize, Serialize};

/// The task state machine owned by the Rust orchestrator (Section 12.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskState {
    /// Request is recorded; no action is implied.
    Created,
    /// Clarifying task scope and allowed data.
    Normalizing,
    /// Collecting current page evidence.
    Observing,
    /// Producing a bounded typed proposal.
    Proposing,
    /// Trusted policy is evaluating the proposal.
    PolicyCheck,
    /// A specifically authorized action is being dispatched.
    Executing,
    /// Evaluating postconditions using evidence.
    Verifying,
    /// Blocked on a typed user decision.
    WaitingForUser,
    /// Manual user control; agent dispatch is stopped.
    UserTakeover,
    /// No new actions are dispatched until explicit Resume.
    Paused,
    /// Policy prevented the proposed action.
    Blocked,
    /// Runtime could not safely continue.
    Failed,
    /// Stopped by user or policy.
    Cancelled,
    /// Orchestration reached a terminal end; this does not imply success.
    Completed,
}

impl TaskState {
    /// Every state, in lifecycle order.
    pub const ALL: [Self; 14] = [
        Self::Created,
        Self::Normalizing,
        Self::Observing,
        Self::Proposing,
        Self::PolicyCheck,
        Self::Executing,
        Self::Verifying,
        Self::WaitingForUser,
        Self::UserTakeover,
        Self::Paused,
        Self::Blocked,
        Self::Failed,
        Self::Cancelled,
        Self::Completed,
    ];

    /// Returns the states reachable from this state, per the normative
    /// transition table. A terminal state returns an empty slice.
    #[must_use]
    pub fn allowed_next(self) -> &'static [Self] {
        match self {
            Self::Created => &[Self::Normalizing, Self::Cancelled, Self::Failed],
            Self::Normalizing => &[
                Self::Observing,
                Self::WaitingForUser,
                Self::Cancelled,
                Self::Failed,
            ],
            Self::Observing => &[
                Self::Proposing,
                Self::WaitingForUser,
                Self::Paused,
                Self::UserTakeover,
                Self::Cancelled,
                Self::Failed,
            ],
            Self::Proposing => &[
                Self::PolicyCheck,
                Self::Observing,
                Self::WaitingForUser,
                Self::Paused,
                Self::UserTakeover,
                Self::Cancelled,
                Self::Failed,
            ],
            Self::PolicyCheck => &[
                Self::Executing,
                Self::Observing,
                Self::WaitingForUser,
                Self::Blocked,
                Self::Paused,
                Self::UserTakeover,
                Self::Cancelled,
                Self::Failed,
            ],
            Self::Executing => &[Self::Verifying, Self::UserTakeover, Self::Failed],
            Self::Verifying => &[
                Self::Observing,
                Self::WaitingForUser,
                Self::Completed,
                Self::Failed,
                Self::Cancelled,
            ],
            Self::WaitingForUser => &[
                Self::PolicyCheck,
                Self::Observing,
                Self::UserTakeover,
                Self::Paused,
                Self::Cancelled,
                Self::Failed,
            ],
            Self::UserTakeover | Self::Paused => {
                &[Self::Observing, Self::Cancelled, Self::Completed]
            }
            Self::Blocked | Self::Failed | Self::Cancelled | Self::Completed => &[],
        }
    }

    /// Returns whether the transition is legal.
    #[must_use]
    pub fn can_transition_to(self, next: Self) -> bool {
        self.allowed_next().contains(&next)
    }

    /// Returns whether no further transition is possible for this task.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Blocked | Self::Failed | Self::Cancelled | Self::Completed
        )
    }

    /// Returns whether the task is actively consuming its execution budget.
    #[must_use]
    pub const fn is_active_work(self) -> bool {
        matches!(
            self,
            Self::Normalizing
                | Self::Observing
                | Self::Proposing
                | Self::PolicyCheck
                | Self::Executing
                | Self::Verifying
        )
    }

    /// Returns the canonical wire code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Created => "CREATED",
            Self::Normalizing => "NORMALIZING",
            Self::Observing => "OBSERVING",
            Self::Proposing => "PROPOSING",
            Self::PolicyCheck => "POLICY_CHECK",
            Self::Executing => "EXECUTING",
            Self::Verifying => "VERIFYING",
            Self::WaitingForUser => "WAITING_FOR_USER",
            Self::UserTakeover => "USER_TAKEOVER",
            Self::Paused => "PAUSED",
            Self::Blocked => "BLOCKED",
            Self::Failed => "FAILED",
            Self::Cancelled => "CANCELLED",
            Self::Completed => "COMPLETED",
        }
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// A rejected lifecycle transition (BA-STATE-002 / Section 12.2).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TaskStateError {
    /// The requested transition is not in the normative table.
    #[error("illegal task transition {from} -> {to}")]
    IllegalTransition {
        /// State the task is currently in.
        from: TaskState,
        /// Requested next state.
        to: TaskState,
    },
    /// A terminal task must not be mutated.
    #[error("task is terminal in {0} and cannot be restarted; start a new task")]
    Terminal(TaskState),
}

/// Exactly one typed reason accompanies `WAITING_FOR_USER` (BA-STATE-002).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WaitReason {
    /// Approval of one bounded reversible action is pending.
    ActionApproval,
    /// An unresolved question needs the user's answer.
    UserInput,
    /// Manual user control is required; no credential is inferred.
    Authentication,
    /// A hard task budget ceiling was reached.
    TaskBudget,
}

impl WaitReason {
    /// Every reason.
    pub const ALL: [Self; 4] = [
        Self::ActionApproval,
        Self::UserInput,
        Self::Authentication,
        Self::TaskBudget,
    ];

    /// Returns the canonical wire code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::ActionApproval => "ACTION_APPROVAL",
            Self::UserInput => "USER_INPUT",
            Self::Authentication => "AUTHENTICATION",
            Self::TaskBudget => "TASK_BUDGET",
        }
    }
}

impl fmt::Display for WaitReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// Runtime policy result describing the consequence class of an action
/// (Section 13.2). This is never chosen by the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionClass {
    /// Inspection or user-intended navigation with no intended mutation.
    Read,
    /// A bounded change with an evidenced compensating operation.
    ReversibleWrite,
    /// A consequential or non-recoverable external side effect.
    Irreversible,
    /// Effect, target, destination or authority cannot be established.
    Unknown,
}

impl ActionClass {
    /// Returns whether the agent may ever execute this class autonomously.
    ///
    /// `IRREVERSIBLE` and `UNKNOWN` are never agent-executed; they require
    /// handoff to the user (Section 13.4).
    #[must_use]
    pub const fn is_agent_executable(self) -> bool {
        matches!(self, Self::Read | Self::ReversibleWrite)
    }

    /// Returns the canonical wire code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Read => "READ",
            Self::ReversibleWrite => "REVERSIBLE_WRITE",
            Self::Irreversible => "IRREVERSIBLE",
            Self::Unknown => "UNKNOWN",
        }
    }
}

impl fmt::Display for ActionClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// Independent verification outcome vocabulary (Section 15.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationOutcome {
    /// All mandatory postconditions hold with fresh evidence.
    VerifiedSuccess,
    /// Positive evidence exists but a required proof is unavailable.
    LikelySuccess,
    /// Evidence is missing, stale, contradictory or interrupted.
    Unknown,
    /// Evidence suggests the postcondition was not achieved.
    LikelyFailure,
    /// Fresh evidence establishes failure.
    VerifiedFailure,
}

impl VerificationOutcome {
    /// Every outcome.
    pub const ALL: [Self; 5] = [
        Self::VerifiedSuccess,
        Self::LikelySuccess,
        Self::Unknown,
        Self::LikelyFailure,
        Self::VerifiedFailure,
    ];

    /// Returns whether only this outcome may use the verified-success colour.
    #[must_use]
    pub const fn is_verified_success(self) -> bool {
        matches!(self, Self::VerifiedSuccess)
    }

    /// Returns the canonical wire code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::VerifiedSuccess => "VERIFIED_SUCCESS",
            Self::LikelySuccess => "LIKELY_SUCCESS",
            Self::Unknown => "UNKNOWN",
            Self::LikelyFailure => "LIKELY_FAILURE",
            Self::VerifiedFailure => "VERIFIED_FAILURE",
        }
    }
}

impl fmt::Display for VerificationOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// Which browser profile a request belongs to (BA-PRIV-001).
///
/// The Agent is disabled in private windows for this release, so policy needs
/// the distinction as a first-class type rather than a boolean.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProfileKind {
    /// The single persistent normal browsing profile.
    Normal,
    /// An ephemeral private context.
    Private,
}

impl ProfileKind {
    /// Returns the canonical wire code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Private => "PRIVATE",
        }
    }
}

impl fmt::Display for ProfileKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// Model capability readiness, independent of task lifecycle (BA-READY-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReadinessState {
    /// No verified model artifact is installed.
    NotInstalled,
    /// An installer is running with measured progress.
    Installing,
    /// A downloaded artifact failed its integrity check.
    IntegrityFailed,
    /// The artifact cannot run on this computer.
    Incompatible,
    /// The runtime is loading the model.
    Loading,
    /// Runner health, schema and benchmark-profile checks passed.
    Ready,
    /// Inference was parked to free memory.
    Parked,
    /// The runner stopped unexpectedly.
    RunnerFailed,
    /// Inference stopped to protect system performance.
    ResourceStopped,
}

impl ReadinessState {
    /// Every readiness state.
    pub const ALL: [Self; 9] = [
        Self::NotInstalled,
        Self::Installing,
        Self::IntegrityFailed,
        Self::Incompatible,
        Self::Loading,
        Self::Ready,
        Self::Parked,
        Self::RunnerFailed,
        Self::ResourceStopped,
    ];

    /// Returns whether tasks may be accepted.
    #[must_use]
    pub const fn accepts_tasks(self) -> bool {
        matches!(self, Self::Ready)
    }

    /// Returns the canonical wire code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NotInstalled => "NOT_INSTALLED",
            Self::Installing => "INSTALLING",
            Self::IntegrityFailed => "INTEGRITY_FAILED",
            Self::Incompatible => "INCOMPATIBLE",
            Self::Loading => "LOADING",
            Self::Ready => "READY",
            Self::Parked => "PARKED",
            Self::RunnerFailed => "RUNNER_FAILED",
            Self::ResourceStopped => "RESOURCE_STOPPED",
        }
    }
}

impl fmt::Display for ReadinessState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_table_matches_the_specification() {
        assert!(TaskState::Created.can_transition_to(TaskState::Normalizing));
        assert!(!TaskState::Created.can_transition_to(TaskState::Executing));
        assert!(TaskState::Executing.can_transition_to(TaskState::Verifying));
        assert!(!TaskState::Executing.can_transition_to(TaskState::Observing));
        assert!(TaskState::WaitingForUser.can_transition_to(TaskState::PolicyCheck));
        assert!(TaskState::Paused.can_transition_to(TaskState::Observing));
        assert!(!TaskState::Paused.can_transition_to(TaskState::Executing));
        for terminal in [
            TaskState::Blocked,
            TaskState::Failed,
            TaskState::Cancelled,
            TaskState::Completed,
        ] {
            assert!(terminal.is_terminal());
            assert!(terminal.allowed_next().is_empty());
        }
    }

    #[test]
    fn only_read_and_reversible_write_are_agent_executable() {
        assert!(ActionClass::Read.is_agent_executable());
        assert!(ActionClass::ReversibleWrite.is_agent_executable());
        assert!(!ActionClass::Irreversible.is_agent_executable());
        assert!(!ActionClass::Unknown.is_agent_executable());
    }

    #[test]
    fn only_verified_success_is_verified() {
        for outcome in VerificationOutcome::ALL {
            assert_eq!(
                outcome.is_verified_success(),
                outcome == VerificationOutcome::VerifiedSuccess
            );
        }
    }

    #[test]
    fn wire_codes_are_stable() {
        assert_eq!(TaskState::WaitingForUser.code(), "WAITING_FOR_USER");
        assert_eq!(WaitReason::TaskBudget.code(), "TASK_BUDGET");
        assert_eq!(ReadinessState::ResourceStopped.code(), "RESOURCE_STOPPED");
    }
}
