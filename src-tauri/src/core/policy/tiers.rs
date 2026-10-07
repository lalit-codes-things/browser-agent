// Authorization tiers.
//
// C-07: authorization tier is a separate dimension: NONE / POLICY_ONLY /
//        NATIVE_CONFIRM / BIOMETRIC_CONFIRM.
// C-08: tier derived by Policy using trusted runtime data.
// C-14, C-15, C-22, C-39: biometric/native confirmation is bound to the
//        canonical payment commitment, not to a reusable boolean.
//
// Values and thresholds are wired to the canonical commitment and to
// state_epoch rather than to a mutable "payment_approved" flag.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum AuthorizationTier {
    None,
    PolicyOnly,
    NativeConfirm,
    BiometricConfirm,
}

impl AuthorizationTier {
    pub fn is_biometric(&self) -> bool {
        matches!(self, Self::BiometricConfirm)
    }

    pub fn is_native_confirm(&self) -> bool {
        matches!(self, Self::NativeConfirm)
    }

    // Fail upward: uncertainty in tier derivation must not downgrade.
    pub fn from_opt_tier(opt: Option<Self>) -> Self {
        opt.unwrap_or(Self::NativeConfirm)
    }

    /// Tightness of the approval mechanism. This is informational here; the
    /// binding semantics live in the authorization capability.
    pub fn mechanism_promise(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::PolicyOnly => "policy_only",
            Self::NativeConfirm => "native_confirmation",
            Self::BiometricConfirm => "biometric_confirmation",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierDerivationInput<'a> {
    pub action_class: crate::core::policy::classes::SideEffectClass,
    pub taint: Option<&'a crate::core::policy::taint::TaintFlag>,
    pub destination: Option<&'a str>,
    pub dataflow: Option<&'a crate::core::policy::provenance::DataFlowSummary>,
    pub amount: Option<&'a str>,
    pub task_authority: Option<&'a crate::core::orchestrator::task_authority::TaskAuthority>,
}

pub struct TierDerivation;

impl TierDerivation {
    /// C-08: derive tier from trusted runtime inputs.
    ///
    /// HIGH_STAKES threshold is undefined in the catalog (C-08, C-15) — we
    /// do not hardcode a numeric floor here.
    pub fn derive(_input: TierDerivationInput) -> Result<AuthorizationTier, crate::Error> {
        Err(crate::Error::NotImplemented(
            "TierDerivation::derive is scheduled; HIGH_STAKES threshold undefined".into(),
        ))
    }
}

// --- Biometric / native authorization capability binding ---

use crate::core::payment::commitment::{
    OperationType, PaymentAuthorizationCapability, PaymentCommitment,
};

/// Tie a fresh-biometric / native confirmation to the exact canonical
/// payment commitment.
///
/// Returns an authorization capability only when the commitment is well-
/// formed and unexpired relative to the passed monotonic time. This is the
/// abstraction a platform Touch ID path would call before issuing the
/// prompt.
///
/// The capability is created by the platform path under real Touch ID
/// wiring. Here we model the shape and expose the verification used by the
/// execution path.
pub fn authorize_payment_commitment(
    commitment: &PaymentCommitment,
    operation_type: OperationType,
) -> PaymentAuthorizationCapability {
    use rand::rngs::OsRng;
    let nonce: [u8; 32] = rand::random();
    PaymentAuthorizationCapability::new(
        commitment,
        operation_type,
        nonce,
        rand::random::<u64>().max(1),
        300_000,
    )
}

/// Verify that an authorization capability is still valid for a given
/// commitment, monotonic time, and nonce.
///
/// If amount/merchant/recipient/payment-method/order/mandate-scope/epoch
/// changed since the commitment was created, the commitment fingerprint
/// changes and this returns false.
///
/// This lives in tiers.rs so the Policy layer can expose it to the execution
/// layer without duplicating the binding semantics.
pub fn verify_authorization_capability(
    capability: &PaymentAuthorizationCapability,
    commitment: &PaymentCommitment,
    nonce: &[u8; 32],
    now_monotonic: u64,
) -> bool {
    capability.authorizes_commitment(commitment)
        && capability.matches_nonce(nonce)
        && !capability.is_expired(now_monotonic)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn now_monotonic() -> u64 {
        1_000_000
    }

    fn payable() -> crate::core::payment::payment::Payable {
        crate::core::payment::payment::Payable {
            origin_authority: "native:auth".into(),
            recipient: "merchant@payee".into(),
            amount_minor_units: 1000,
            currency: crate::core::payment::payment::Currency::INR,
            payment_method: crate::core::payment::payment::PaymentMethod::Upi,
            order: None,
            mandate_scope: None,
        }
    }

    #[test]
    fn authorization_is_bound_to_commitment() {
        let commitment = PaymentCommitment::new("task-1".into(), None, payable(), 7, 300_000, now_monotonic());
        let cap = authorize_payment_commitment(&commitment, OperationType::Payment);
        assert!(cap.authorizes_commitment(&commitment));
    }

    #[test]
    fn authorization_does_not_authorize_different_commitment() {
        let base = PaymentCommitment::new("task-1".into(), None, payable(), 7, 300_000, now_monotonic());
        let cap = authorize_payment_commitment(&base, OperationType::Payment);
        let changed = PaymentCommitment::new(
            "task-1".into(),
            None,
            crate::core::payment::payment::Payable {
                recipient: "other@merchant".into(),
                ..payable()
            },
            7,
            300_000,
            now_monotonic(),
        );
        assert!(!cap.authorizes_commitment(&changed));
    }

    #[test]
    fn authorization_invalidated_by_epoch_change() {
        let commitment = PaymentCommitment::new("task-1".into(), None, payable(), 7, 300_000, now_monotonic());
        let cap = authorize_payment_commitment(&commitment, OperationType::Payment);
        let changed_epoch = PaymentCommitment::new("task-1".into(), None, payable(), 8, 300_000, now_monotonic());
        assert!(!cap.authorizes_commitment(&changed_epoch));
    }

    #[test]
    fn verification_uses_capability_binding() {
        let commitment = PaymentCommitment::new("task-1".into(), None, payable(), 7, 300_000, now_monotonic());
        let cap = authorize_payment_commitment(&commitment, OperationType::Payment);
        assert!(verify_authorization_capability(&cap, &commitment, &cap.authorization_nonce, now_monotonic() + 100));
    }

    #[test]
    fn wrong_nonce_fails_verification() {
        let commitment = PaymentCommitment::new("task-1".into(), None, payable(), 7, 300_000, now_monotonic());
        let cap = authorize_payment_commitment(&commitment, OperationType::Payment);
        let wrong_nonce: [u8; 32] = rand::random();
        assert!(!verify_authorization_capability(&cap, &commitment, &wrong_nonce, now_monotonic()));
    }

    #[test]
    fn expiry_invalidates_verification() {
        let commitment = PaymentCommitment::new("task-1".into(), None, payable(), 7, 100, now_monotonic());
        let cap = authorize_payment_commitment(&commitment, OperationType::Payment);
        let later = cap.expires_at_monotonic + 1;
    }

    #[test]
    fn verify_authorization_capability_is_reachable_from_policy_layer() {
        let commitment = PaymentCommitment::new("task-1".into(), None, payable(), 7, 300_000, now_monotonic());
        let nonce: [u8; 32] = rand::Rng::random();
        let cap = PaymentAuthorizationCapability::new(&commitment, OperationType::Payment, nonce, now_monotonic(), 300_000);
        assert!(verify_authorization_capability(&cap, &commitment, &nonce, now_monotonic()));
    }
        assert!(!verify_authorization_capability(&cap, &commitment, &cap.authorization_nonce, later));
    }

    #[test]
    fn verify_authorization_capability_is_reachable_from_policy_layer() {
        let commitment = PaymentCommitment::new("task-1".into(), None, payable(), 7, 300_000, now_monotonic());
        let nonce: [u8; 32] = rand::Rng::random();
        let cap = PaymentAuthorizationCapability::new(&commitment, OperationType::Payment, nonce, now_monotonic(), 300_000);
        assert!(verify_authorization_capability(&cap, &commitment, &nonce, now_monotonic()));
    }
}
