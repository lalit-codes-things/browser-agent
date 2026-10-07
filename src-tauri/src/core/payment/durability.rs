// Payment durability.
//
// C-39, C-11, C-92, C-100: an externally mutating payment submission
// crosses an irreversible boundary once it is dispatched. Before that
// boundary is considered committed, enough durable state must be persisted
// to classify the operation on crash recovery.
//
// Persisted durable submission state MUST exist BEFORE the external
// submission boundary is considered committed.
//
// Crash outcomes:
//   BEFORE submission boundary  -> classify per pre-submission recovery
//                                   rules (Preparing/Reconciling/Ready/...
//                                   depending on how far the task advanced).
//   AFTER submission boundary   -> UNKNOWN. Then require a human checkpoint
//                                   / fresh verification. NEVER automatically
//                                   retry merely because the app restarted.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentDurableState {
    /// Prepared locally, not yet durably persisted.
    Prepared,
    /// Reconciled against task authority; durable record exists but
    /// external dispatch has not happened.
    Reconciled,
    /// Ready to submit; durable record exists.
    Ready,
    /// Submitted to an external ecosystem (example: upi:// link opened via
    /// user action, or payment service invoked after explicit authorization);
    /// durable record exists. The external result is not yet observed.
    Submitted,
    /// External ecosystem authentication is outstanding.
    WaitingForExternalAuth,
    /// External result observed and being verified.
    Processing,
    /// External result observed but verification inconclusive.
    Unknown,
    /// Verified success is only reachable through independent verification.
    VerifiedSuccess,
    /// Verified failure.
    VerifiedFailure,
}

// Idempotency key bound to the durable submission record.
//
// The idempotency key is a double-charge protection mechanism. It is NOT
// permission for autonomous retry of an unresolved payment after a crash;
// that remains prohibited (unknown -> human checkpoint -> verification).
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

/// Durable record persisted before the external submission boundary is
/// considered committed.
///
/// On crash, this record is the substrate for recovery:
///   - If persisted and NOT yet submitted externally -> classify per
///     PaymentDurableState (which remains Reconciled/Ready until the
///     submission endpoint is crossed in the durable state).
///   - If persisted AND submitted -> recovered state is Submitted, and then
///     the runtime must move to Unknown if the post-submission result is
///     inconclusive, then require human checkpoint + fresh verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurablePaymentRecord {
    pub durable_id: String,
    pub task_id: String,
    pub payment_commitment_hash: [u8; 32],
    pub idempotency_key: IdempotencyKey,
    pub durable_state: PaymentDurableState,
    /// Trusted origin/authority known to the task, not page-derived.
    pub origin_authority: String,
    pub recipient: String,
    pub amount_minor_units: u64,
    pub currency: crate::core::payment::payment::Currency,
    pub payment_method: crate::core::payment::payment::PaymentMethod,
    pub order: Option<crate::core::payment::payment::OrderIdentity>,
    pub submission_epoch: u64,
    pub created_at_monotonic: u64,
}

impl DurablePaymentRecord {
    /// Persisted before the submit-to-external-ecosystem boundary is crossed.
    pub fn submitted_before_dispatch(&self) -> bool {
        self.durable_state == PaymentDurableState::Reconciled
            || self.durable_state == PaymentDurableState::Ready
    }

    /// After this state is reached and persisted, the operation has crossed
    /// the submission boundary. Post-crash recovery treats this as UNKNOWN
    /// first if the result is inconclusive.
    pub fn submitted_after_dispatch(&self) -> bool {
        matches!(
            self.durable_state,
            PaymentDurableState::Submitted
                | PaymentDurableState::WaitingForExternalAuth
                | PaymentDurableState::Processing
        )
    }

    /// Durable records for submitted-then-unresolved payment MUST NOT be
    /// interpreted as retryable success by the runtime.
    pub fn submitted_and_unresolved(&self) -> bool {
        self.durable_state == PaymentDurableState::Submitted
            || self.durable_state == PaymentDurableState::Unknown
            || self.durable_state == PaymentDurableState::Processing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::payment::payment::Payable;

    #[test]
    fn before_submission_boundary_is_not_submitted() {
        let rec = DurablePaymentRecord {
            durable_id: "dr-1".into(),
            task_id: "t-1".into(),
            payment_commitment_hash: Default::default(),
            idempotency_key: IdempotencyKey::new("ik-1".into(), 1_000_000),
            durable_state: PaymentDurableState::Ready,
            origin_authority: "native:auth".into(),
            recipient: "merchant@payee".into(),
            amount_minor_units: 1000,
            currency: crate::core::payment::payment::Currency::INR,
            payment_method: Payable::default().payment_method,
            order: None,
            submission_epoch: 7,
            created_at_monotonic: 1_000_000,
        };
        assert!(rec.submitted_before_dispatch());
        assert!(!rec.submitted_after_dispatch());
        assert!(!rec.submitted_and_unresolved());
    }

    #[test]
    fn after_dispatch_is_unresolved() {
        let rec = DurablePaymentRecord {
            durable_id: "dr-1".into(),
            task_id: "t-1".into(),
            payment_commitment_hash: Default::default(),
            idempotency_key: IdempotencyKey::new("ik-1".into(), 1_000_000),
            durable_state: PaymentDurableState::Submitted,
            origin_authority: "native:auth".into(),
            recipient: "merchant@payee".into(),
            amount_minor_units: 1000,
            currency: crate::core::payment::payment::Currency::INR,
            payment_method: Payable::default().payment_method,
            order: None,
            submission_epoch: 7,
            created_at_monotonic: 1_000_000,
        };
        assert!(rec.submitted_after_dispatch());
        assert!(rec.submitted_and_unresolved());
    }

    #[test]
    fn unknown_is_unresolved() {
        let rec = DurablePaymentRecord {
            durable_id: "dr-1".into(),
            task_id: "t-1".into(),
            payment_commitment_hash: Default::default(),
            idempotency_key: IdempotencyKey::new("ik-1".into(), 1_000_000),
            durable_state: PaymentDurableState::Unknown,
            origin_authority: "native:auth".into(),
            recipient: "merchant@payee".into(),
            amount_minor_units: 1000,
            currency: crate::core::payment::payment::Currency::INR,
            payment_method: Payable::default().payment_method,
            order: None,
            submission_epoch: 7,
            created_at_monotonic: 1_000_000,
        };
        assert!(rec.submitted_and_unresolved());
    }

    #[test]
    fn idempotency_key_with_expiry() {
        let ik = IdempotencyKey::new("ik-1".into(), 100);
        assert!(ik.alive_at(50));
        assert!(!ik.alive_at(100));
        // Exactly at expiry the key is not alive.
        assert!(!ik.alive_at(100));
        assert!(!ik.alive_at(200));
    }
}
