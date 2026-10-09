// Generic durable action state.
//
// A consequential action crosses an irreversible boundary once it is
// dispatched externally. Before that boundary is considered committed,
// enough durable state must be persisted to classify the operation on crash
// recovery.
//
// Crash outcomes:
//   BEFORE submission boundary -> classify per pre-submission recovery rules
//   AFTER submission boundary  -> UNKNOWN, then require human checkpoint +
//                                fresh verification; never auto-retry merely
//                                because the app restarted.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DurableActionState {
    Prepared,
    Reconciled,
    Ready,
    Submitted,
    WaitingForExternalAuth,
    Processing,
    Unknown,
    VerifiedSuccess,
    VerifiedFailure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotencyKey {
    pub key: String,
    pub expires_at_monotonic: u64,
}

impl IdempotencyKey {
    pub fn new(key: String, expires_at_monotonic: u64) -> Self {
        Self { key, expires_at_monotonic }
    }

    pub fn alive_at(&self, now_monotonic: u64) -> bool {
        now_monotonic < self.expires_at_monotonic
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableActionRecord {
    pub durable_id: String,
    pub task_id: String,
    pub action_commitment_hash: [u8; 32],
    pub idempotency_key: IdempotencyKey,
    pub durable_state: DurableActionState,
    pub origin_authority: String,
    pub destination: String,
    pub submission_epoch: u64,
    pub created_at_monotonic: u64,
}

impl DurableActionRecord {
    pub fn submitted_before_dispatch(&self) -> bool {
        self.durable_state == DurableActionState::Reconciled
            || self.durable_state == DurableActionState::Ready
    }

    pub fn submitted_after_dispatch(&self) -> bool {
        matches!(
            self.durable_state,
            DurableActionState::Submitted
                | DurableActionState::WaitingForExternalAuth
                | DurableActionState::Processing
        )
    }

    pub fn submitted_and_unresolved(&self) -> bool {
        self.durable_state == DurableActionState::Submitted
            || self.durable_state == DurableActionState::Unknown
            || self.durable_state == DurableActionState::Processing
    }
}

/// Classify a durable action record after a crash / restart / resume.
///
/// No automatic retry is ever produced here.
pub fn classify_durable_after_crash(rec: &DurableActionRecord) -> DurableActionRecoveryOutcome {
    if rec.submitted_before_dispatch() {
        return DurableActionRecoveryOutcome::PreSubmission;
    }
    if rec.submitted_after_dispatch() || rec.submitted_and_unresolved() {
        return DurableActionRecoveryOutcome::UnknownRequiresHumanCheckpoint;
    }
    DurableActionRecoveryOutcome::PreSubmission
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DurableActionRecoveryOutcome {
    PreSubmission,
    UnknownRequiresHumanCheckpoint,
    NotYetVerified,
}

impl DurableActionState {
    pub fn recovery_path(&self) -> DurableActionRecoveryOutcome {
        match self {
            Self::Prepared | Self::Reconciled | Self::Ready => {
                DurableActionRecoveryOutcome::UnknownRequiresHumanCheckpoint
            }
            Self::Submitted | Self::WaitingForExternalAuth | Self::Processing | Self::Unknown => {
                DurableActionRecoveryOutcome::UnknownRequiresHumanCheckpoint
            }
            Self::VerifiedSuccess | Self::VerifiedFailure => DurableActionRecoveryOutcome::NotYetVerified,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(state: DurableActionState) -> DurableActionRecord {
        DurableActionRecord {
            durable_id: "dr-1".into(),
            task_id: "t-1".into(),
            action_commitment_hash: Default::default(),
            idempotency_key: IdempotencyKey::new("ik-1".into(), 1_000_000),
            durable_state: state,
            origin_authority: "native:auth".into(),
            destination: "merchant@payee".into(),
            submission_epoch: 7,
            created_at_monotonic: 1_000_000,
        }
    }

    #[test]
    fn submitted_after_crash_is_unknown() {
        let rec = rec(DurableActionState::Submitted);
        assert_eq!(
            classify_durable_after_crash(&rec),
            DurableActionRecoveryOutcome::UnknownRequiresHumanCheckpoint
        );
        assert_eq!(
            rec.durable_state.recovery_path(),
            DurableActionRecoveryOutcome::UnknownRequiresHumanCheckpoint
        );
    }

    #[test]
    fn waiting_for_external_auth_is_unknown_on_resume() {
        let rec = rec(DurableActionState::WaitingForExternalAuth);
        assert_eq!(
            classify_durable_after_crash(&rec),
            DurableActionRecoveryOutcome::UnknownRequiresHumanCheckpoint
        );
    }

    #[test]
    fn ready_is_pre_submission() {
        let rec = rec(DurableActionState::Ready);
        assert_eq!(
            classify_durable_after_crash(&rec),
            DurableActionRecoveryOutcome::PreSubmission
        );
    }

    #[test]
    fn reconciled_is_pre_submission() {
        let rec = rec(DurableActionState::Reconciled);
        assert_eq!(
            classify_durable_after_crash(&rec),
            DurableActionRecoveryOutcome::PreSubmission
        );
    }

    #[test]
    fn unknown_is_unknown_requires_human_checkpoint() {
        let rec = rec(DurableActionState::Unknown);
        assert_eq!(
            classify_durable_after_crash(&rec),
            DurableActionRecoveryOutcome::UnknownRequiresHumanCheckpoint
        );
        assert_eq!(
            rec.durable_state.recovery_path(),
            DurableActionRecoveryOutcome::UnknownRequiresHumanCheckpoint
        );
    }

    #[test]
    fn idempotency_key_with_expiry() {
        let ik = IdempotencyKey::new("ik-1".into(), 100);
        assert!(ik.alive_at(50));
        assert!(!ik.alive_at(100));
        assert!(!ik.alive_at(200));
    }
}
