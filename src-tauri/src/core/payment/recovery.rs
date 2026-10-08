// Payment recovery.
//
// C-39, C-92, C-100, C-11: after a crash or ambiguous external result,
//        unresolved payment state resolves to UNKNOWN and requires a human
//        checkpoint plus fresh independent verification. The runtime never
//        automatically retries an unresolved payment simply because the
//        application restarted or the UI did not change.
//
// The idempotency key protects against double charges at the external
// ecosystem, not at the agent runtime. It does not authorize retry.

use crate::core::payment::durability::{DurablePaymentRecord, IdempotencyKey, PaymentDurableState};
use crate::core::payment::payment::Payable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentRecoveryOutcome {
    /// Pre-submission durable state: operation can be classified per the
    /// policy/pre-submission recovery rules without treating it as committed.
    PreSubmission,
    /// Submitted durably but post-submission result is inconclusive. The
    /// runtime MUST resolve to UNKNOWN and then require human checkpoint +
    /// fresh independent verification. It never auto-retries.
    UnknownRequiresHumanCheckpoint,
    /// Verified success is only reachable through the independent
    /// verification hierarchy; the runtime does not emit this on resume.
    NotYetVerified,
}

/// Classify a durable payment record after a crash / restart / resume.
///
/// This is the authoritative recovery decision for payment state.
/// No automatic retry is ever produced here.
pub fn classify_durable_after_crash(rec: &DurablePaymentRecord) -> PaymentRecoveryOutcome {
    if rec.submitted_before_dispatch() {
        return PaymentRecoveryOutcome::PreSubmission;
    }
    if rec.submitted_after_dispatch() || rec.submitted_and_unresolved() {
        return PaymentRecoveryOutcome::UnknownRequiresHumanCheckpoint;
    }
    // Reconciled states fall through to pre-submission handling.
    PaymentRecoveryOutcome::PreSubmission
}

impl PaymentDurableState {
    /// Resolution path after a crash / resume for each durable state.
    ///
    /// Submitted states and unresolved states become UNKNOWN + human
    /// checkpoint; they are never treated as successful on resume.
    pub fn recovery_path(&self) -> PaymentRecoveryOutcome {
        match self {
            PaymentDurableState::Prepared
            | PaymentDurableState::Reconciled
            | PaymentDurableState::Ready
            | PaymentDurableState::Processing => {
                // Processing is treated as unresolved until verified.
                PaymentRecoveryOutcome::UnknownRequiresHumanCheckpoint
            }
            PaymentDurableState::Submitted
            | PaymentDurableState::WaitingForExternalAuth => {
                PaymentRecoveryOutcome::UnknownRequiresHumanCheckpoint
            }
            PaymentDurableState::Unknown => {
                PaymentRecoveryOutcome::UnknownRequiresHumanCheckpoint
            }
            PaymentDurableState::VerifiedSuccess | PaymentDurableState::VerifiedFailure => {
                // Only reachable via verification; treated here as
                // "already verified" only if the verification path actually
                // produced it. Resume must not invent this.
                PaymentRecoveryOutcome::NotYetVerified
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::payment::payment::Payable;

    fn base_recipient() -> (String, String, PaymentDurableState) {
        (
            "native:auth".into(),
            "merchant@payee".into(),
            PaymentDurableState::Submitted,
        )
    }

    fn rec(durable_state: PaymentDurableState) -> DurablePaymentRecord {
        let (origin, recipient, _) = base_recipient();
        DurablePaymentRecord {
            durable_id: "dr-1".into(),
            task_id: "t-1".into(),
            payment_commitment_hash: Default::default(),
            idempotency_key: IdempotencyKey::new("ik-1".into(), 1_000_000),
            durable_state,
            origin_authority: origin,
            recipient,
            amount_minor_units: 1000,
            currency: Payable::default().currency,
            payment_method: Payable::default().payment_method,
            order: None,
            submission_epoch: 7,
            created_at_monotonic: 1_000_000,
        }
    }

    #[test]
    fn submitted_after_crash_is_unknown() {
        let rec = rec(PaymentDurableState::Submitted);
        assert_eq!(
            classify_durable_after_crash(&rec),
            PaymentRecoveryOutcome::UnknownRequiresHumanCheckpoint
        );
        assert_eq!(
            rec.durable_state.recovery_path(),
            PaymentRecoveryOutcome::UnknownRequiresHumanCheckpoint
        );
    }

    #[test]
    fn waiting_for_external_auth_is_unknown_on_resume() {
        let rec = rec(PaymentDurableState::WaitingForExternalAuth);
        assert_eq!(
            classify_durable_after_crash(&rec),
            PaymentRecoveryOutcome::UnknownRequiresHumanCheckpoint
        );
    }

    #[test]
    fn pre_submission_is_pre_submission() {
        let rec = rec(PaymentDurableState::Ready);
        assert_eq!(classify_durable_after_crash(&rec), PaymentRecoveryOutcome::PreSubmission);
    }

    #[test]
    fn reconciled_is_pre_submission() {
        let rec = rec(PaymentDurableState::Reconciled);
        assert_eq!(classify_durable_after_crash(&rec), PaymentRecoveryOutcome::PreSubmission);
    }

    #[test]
    fn unknown_is_unknown_requires_human_checkpoint() {
        let rec = rec(PaymentDurableState::Unknown);
        assert_eq!(
            classify_durable_after_crash(&rec),
            PaymentRecoveryOutcome::UnknownRequiresHumanCheckpoint
        );
        assert_eq!(
            rec.durable_state.recovery_path(),
            PaymentRecoveryOutcome::UnknownRequiresHumanCheckpoint
        );
    }

    #[test]
    fn submitted_requires_human_checkpoint() {
        let rec = rec(PaymentDurableState::Submitted);
        assert_eq!(
            classify_durable_after_crash(&rec),
            PaymentRecoveryOutcome::UnknownRequiresHumanCheckpoint
        );
        assert_eq!(
            rec.durable_state.recovery_path(),
            PaymentRecoveryOutcome::UnknownRequiresHumanCheckpoint
        );
    }
}
