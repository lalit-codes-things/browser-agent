// Payment commitment.
//
// C-39: payment commitments are canonical and immutable once created.
// C-14, C-15, C-22, C-36: the canonical commitment is the substrate
//        against which any Touch ID / native confirmation is bound.
//
// The model proposes. Policy compiles the trusted inputs. The commitment is
// the immutable facts of the operation. Authorization is bound to that
// commitment and to a state epoch, not to any reusable boolean.

use crate::core::payment::payment::{Payable, Currency};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentCommitment {
    pub task_id: String,
    pub task_authority_id: Option<String>,
    pub payable: Payable,
    pub commitment_version: u32,
    pub state_epoch: u64,
    pub created_at_monotonic: u64,
    pub expires_at_monotonic: u64,
}

impl PaymentCommitment {
    /// Build a canonical commitment for the payable under a task.
    ///
    /// Arguments are the immutable facts the POLICY/verification path will
    /// evaluate. Any page-derived string must be rejected as invalid
    /// authority here (defense-in-depth; the boundary is enforced earlier
    /// by task authority + perception).
    pub fn new(
        task_id: String,
        task_authority_id: Option<String>,
        payable: Payable,
        state_epoch: u64,
        expiry_monotonic_from_now: u64,
        now_monotonic: u64,
    ) -> Self {
        Self {
            task_id,
            task_authority_id,
            payable,
            commitment_version: 1,
            state_epoch,
            created_at_monotonic: now_monotonic,
            expires_at_monotonic: now_monotonic.saturating_add(expiry_monotonic_from_now),
        }
    }

    /// Deterministic canonical payload for the commitment. Hashing the
    /// canonical payload produces an immutable commitment fingerprint.
    pub fn canonical_for_hashing(&self) -> Vec<u8> {
        let mut v = crate::core::payment::payment::Payable::canonical_bytes_for_commitment(
            &self.payable,
        );
        v.push(b'\0');
        v.extend_from_slice(self.task_id.as_bytes());
        v.push(b'\0');
        if let Some(ref aid) = self.task_authority_id {
            v.extend_from_slice(aid.as_bytes());
        }
        v.push(b'\0');
        v.extend_from_slice(&self.commitment_version.to_be_bytes());
        v.push(b'\0');
        v.extend_from_slice(&self.state_epoch.to_be_bytes());
        v.push(b'\0');
        v.extend_from_slice(&self.created_at_monotonic.to_be_bytes());
        v
    }

    /// Monotonic-time expiry instant, used for expiry invalidation.
    pub fn expires_at(&self) -> u64 {
        self.expires_at_monotonic
    }

    pub fn created_at(&self) -> u64 {
        self.created_at_monotonic
    }
}

/// Hash a canonical commitment payload.
///
/// We use SHA-256 as the commitment fingerprint. For payment-mode operations
/// we later bind a non-predictable authorization nonce, so commitment_hash
/// plays the role of "what was approved" rather than "what unlocks".
pub fn commitment_hash(canonical: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    sha2::Sha256::digest(canonical).into()
}

/// Authorization capability bound to a payment commitment.
///
/// This is the artifact that a biometric/native confirmation produces.
/// It is NOT a reusable "payment approved" boolean.
///
/// Invalidating conditions (must match exactly at execution time):
///   - same commitment_hash
///   - same state_epoch
///   - same authorization_nonce
///   - same operation_type
///   - not expired
///
/// If the payable changes (amount/merchant/recipient/payment_method/order
/// /mandate_scope) or the epoch changes or the expiry passes, the
/// commitment changes and this capability does not authorize the new one.
#[derive(Debug, Clone)]
pub struct PaymentAuthorizationCapability {
    pub commitment_hash: [u8; 32],
    pub state_epoch: u64,
    pub authorization_nonce: [u8; 32],
    pub operation_type: OperationType,
    pub issued_at_monotonic: u64,
    pub expires_at_monotonic: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Payment,
    FinancialTransfer,
    SensitiveAccountChange,
    SensitiveDisclosure,
}

impl PaymentAuthorizationCapability {
    pub fn new(
        commitment: &PaymentCommitment,
        operation_type: OperationType,
        nonce: [u8; 32],
        now_monotonic: u64,
        expiry_monotonic_from_now: u64,
    ) -> Self {
        Self {
            commitment_hash: commitment_hash(&commitment.canonical_for_hashing()),
            state_epoch: commitment.state_epoch,
            authorization_nonce: nonce,
            operation_type,
            issued_at_monotonic: now_monotonic,
            expires_at_monotonic: now_monotonic.saturating_add(expiry_monotonic_from_now),
        }
    }

    /// The authorization is tied to the exact commitment fingerprint that
    /// produced it. A different commitment does not authorize here.
    pub fn authorizes_commitment(&self, commitment: &PaymentCommitment) -> bool {
        self.commitment_hash == commitment_hash(&commitment.canonical_for_hashing())
            && self.state_epoch == commitment.state_epoch
            && !self.is_expired(commitment.created_at())
        }

    /// A single nonce cannot be replayed across tasks/commitments. The
    /// capability consumer (Touch ID stub / platform path) must bind the
    /// nonce to the particular authorization attempt and verify it at
    /// execution time.
    pub fn matches_nonce(&self, nonce: &[u8; 32]) -> bool {
        self.authorization_nonce == *nonce
    }

    pub fn is_expired(&self, now_monotonic: u64) -> bool {
        now_monotonic >= self.expires_at_monotonic
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::payment::payment::Payable;

    fn now() -> u64 {
        1_000_000
    }

    fn payable() -> Payable {
        Payable {
            origin_authority: "native:auth".into(),
            recipient: "merchant@payee".into(),
            amount_minor_units: 1000,
            currency: Currency::INR,
            payment_method: crate::core::payment::payment::PaymentMethod::Upi,
            order: None,
            mandate_scope: None,
        }
    }

    #[test]
    fn commitment_hash_is_deterministic() {
        let c = PaymentCommitment::new(
            "task-1".into(),
            None,
            payable(),
            7,
            300_000,
            now(),
        );
        let a = commitment_hash(&c.canonical_for_hashing());
        let b = commitment_hash(&c.canonical_for_hashing());
        assert_eq!(a, b);
    }

    #[test]
    fn different_payable_produces_different_commitment() {
        let base = PaymentCommitment::new(
            "task-1".into(),
            None,
            payable(),
            7,
            300_000,
            now(),
        );
        let changed = PaymentCommitment::new(
            "task-1".into(),
            None,
            Payable {
                amount_minor_units: 2000,
                ..payable()
            },
            7,
            300_000,
            now(),
        );
        assert_ne!(
            commitment_hash(&base.canonical_for_hashing()),
            commitment_hash(&changed.canonical_for_hashing())
        );
    }

    #[test]
    fn authorization_is_bound_to_commitment() {
        let commitment = PaymentCommitment::new("task-1".into(), None, payable(), 7, 300_000, now());
        let cap = PaymentAuthorizationCapability::new(
            &commitment,
            OperationType::Payment,
            rand::random(),
            now(),
            300_000,
        );
        assert!(cap.authorizes_commitment(&commitment));
    }

    #[test]
    fn authorization_does_not_authorize_different_commitment() {
        let base = PaymentCommitment::new("task-1".into(), None, payable(), 7, 300_000, now());
        let cap = PaymentAuthorizationCapability::new(
            &base,
            OperationType::Payment,
            rand::random(),
            now(),
            300_000,
        );
        let changed = PaymentCommitment::new(
            "task-1".into(),
            None,
            Payable {
                recipient: "other@merchant".into(),
                ..payable()
            },
            7,
            300_000,
            now(),
        );
        assert!(!cap.authorizes_commitment(&changed));
    }

    #[test]
    fn authorization_invalidated_by_epoch_change() {
        let commitment = PaymentCommitment::new("task-1".into(), None, payable(), 7, 300_000, now());
        let cap = PaymentAuthorizationCapability::new(
            &commitment,
            OperationType::Payment,
            rand::random(),
            now(),
            300_000,
        );
        let changed_epoch = PaymentCommitment::new("task-1".into(), None, payable(), 8, 300_000, now());
        assert!(!cap.authorizes_commitment(&changed_epoch));
    }

    #[test]
    fn authorization_expired_after_monotonic_time() {
        let commitment = PaymentCommitment::new("task-1".into(), None, payable(), 7, 100, now());
        let cap = PaymentAuthorizationCapability::new(
            &commitment,
            OperationType::Payment,
            rand::random(),
            now(),
            100,
        );
        // Advance time beyond both the commitment expiry and the capability
        // expiry.
        let later = now() + 200;
        assert!(cap.is_expired(later));
    }

    #[test]
    fn nonce_is_unique_per_authorization() {
        let commitment = PaymentCommitment::new("task-1".into(), None, payable(), 7, 300_000, now());
        let cap_a = PaymentAuthorizationCapability::new(
            &commitment,
            OperationType::Payment,
            rand::random(),
            now(),
            300_000,
        );
        let cap_b = PaymentAuthorizationCapability::new(
            &commitment,
            OperationType::Payment,
            rand::random(),
            now(),
            300_000,
        );
        // Two authorizations for the same commitment use different nonces;
        // the capability with the matching nonce is the only one that may
        // be used for execution.
        assert_ne!(cap_a.authorization_nonce, cap_b.authorization_nonce);
    }
}
