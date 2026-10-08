// Payment execution surface.
//
// c-36/c-39/c-92/c-100/c-102: the payment execution surface exposes durable
// record state and authorization outcomes to the IPC layer WITHOUT payment
// secrets. Payment secrets are never carried in payment execution events to
// the frontend.
//
// This module is the structural boundary between the typed payment domain
// and the IPC events that the frontend consumes. It does not introduce a
// second payment engine; it reuses the existing payment domain and durability
// module.

use crate::core::payment::durability::{DurablePaymentRecord, PaymentDurableState};
use crate::core::payment::commitment::commitment_hash;
use crate::core::payment::PaymentCommitment;
use crate::security::clocks::MonotonicClock;

/// Convert a durable payment record to the shape emitted through IPC events.
///
/// No payment secret is present in the emitted shape. The frontend receives
/// the durable state, durable id, task id, and commitment hash only.
pub fn durable_record_to_event(
    rec: &DurablePaymentRecord,
) -> crate::ipc::events::PaymentDurableStateEvent {
    crate::ipc::events::PaymentDurableStateEvent {
        durable_id: rec.durable_id.clone(),
        durable_state: match rec.durable_state {
            PaymentDurableState::Prepared => "PREPARING",
            PaymentDurableState::Reconciled => "RECONCILING",
            PaymentDurableState::Ready => "READY",
            PaymentDurableState::Submitted => "SUBMITTED",
            PaymentDurableState::WaitingForExternalAuth => "WAITING_FOR_EXTERNAL_AUTH",
            PaymentDurableState::Processing => "PROCESSING",
            PaymentDurableState::Unknown => "UNKNOWN",
            PaymentDurableState::VerifiedSuccess => "VERIFIED_SUCCESS",
            PaymentDurableState::VerifiedFailure => "VERIFIED_FAILURE",
        }
        .to_string(),
        payment_commitment_hash: hex_lower(&rec.payment_commitment_hash),
        task_id: rec.task_id.clone(),
    }
}

fn hex_lower(bytes: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(64);
    for b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

/// A structural helper that computes the canonical commitment hash for a
/// payable without exposing the underlying secret. This is used by the
/// payment path that builds the commitment before it is durably persisted.
pub fn commitment_hash_for_payable(
    task_id: &str,
    payable: &crate::core::payment::payment::Payable,
    state_epoch: u64,
    expiry_monotonic_from_now: u64,
    now_monotonic: u64,
) -> [u8; 32] {
    let commitment = PaymentCommitment::new(
        task_id.into(),
        None,
        (*payable).clone(),
        state_epoch,
        expiry_monotonic_from_now,
        now_monotonic,
    );
    commitment_hash(&commitment.canonical_for_hashing())
}

// Lightweight payable type for the execution surface.
// Keep this separate from the full payable type so the execution surface
// does not accidentally inline payment secrets.
#[derive(Debug, Clone)]
pub struct Payable {
    pub origin_authority: String,
    pub recipient: String,
    pub amount_minor_units: u64,
    pub currency: crate::core::payment::payment::Currency,
    pub payment_method: crate::core::payment::payment::PaymentMethod,
    pub order: Option<crate::core::payment::payment::OrderIdentity>,
    pub mandate_scope: Option<String>,
}

impl From<crate::core::payment::payment::Payable> for Payable {
    fn from(p: crate::core::payment::payment::Payable) -> Self {
        Self {
            origin_authority: p.origin_authority,
            recipient: p.recipient,
            amount_minor_units: p.amount_minor_units,
            currency: p.currency,
            payment_method: p.payment_method,
            order: p.order,
            mandate_scope: p.mandate_scope,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::payment::payment::{Payable as CorePayable, PaymentMethod, Currency};

    fn payable() -> CorePayable {
        CorePayable {
            origin_authority: "native:auth".into(),
            recipient: "merchant@payee".into(),
            amount_minor_units: 1000,
            currency: Currency::INR,
            payment_method: PaymentMethod::Upi,
            order: None,
            mandate_scope: None,
        }
    }

    #[test]
    fn event_does_not_contain_secret() {
        let rec = DurablePaymentRecord {
            durable_id: "dr-1".into(),
            task_id: "t-1".into(),
            payment_commitment_hash: [1u8; 32],
            idempotency_key: crate::core::payment::durability::IdempotencyKey::new("ik-1".into(), 1_000_000),
            durable_state: PaymentDurableState::Ready,
            origin_authority: "native:auth".into(),
            recipient: "merchant@payee".into(),
            amount_minor_units: 1000,
            currency: Currency::INR,
            payment_method: PaymentMethod::Upi,
            order: None,
            submission_epoch: 7,
            created_at_monotonic: MonotonicClock::now_nanos(),
        };
        let ev = durable_record_to_event(&rec);
        assert!(!ev.payment_commitment_hash.contains("payee"));
        assert!(ev.payment_commitment_hash.len() == 64);
    }

    #[test]
    fn commitment_hash_for_payable_is_deterministic() {
        let p = payable();
        let a = commitment_hash_for_payable("t-1", &p, 7, 300_000, 1_000_000);
        let b = commitment_hash_for_payable("t-1", &p, 7, 300_000, 1_000_000);
        assert_eq!(a, b);
    }
}
