// Authorization tiers.
//
// C-07: authorization tier is a separate dimension: NONE / POLICY_ONLY /
//        NATIVE_CONFIRM / BIOMETRIC_CONFIRM.
// C-08: tier derived by Policy using trusted runtime data.
// C-14, C-15, C-22, C-39: biometric/native confirmation is bound to the
//        canonical action commitment, not to a reusable boolean.
//
// Values and thresholds are wired to the canonical commitment and to
// state_epoch rather than to a mutable "approved" flag.

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
        opt.unwrap_or(Self::BiometricConfirm)
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

// Borrowing derivation input: the input references trusted runtime data and
// is never owned, serialized, or reconstructed from page text.
#[derive(Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use super::AuthorizationTier;
    use crate::core::action::ActionAuthorizationCapability;
    use crate::core::action::OperationType;
    use crate::core::action::commitment::ActionCommitment;

    #[allow(unused_imports, non_upper_case_globals)]
    const _tier_test_imports: () = { let _ = (AuthorizationTier::None,); };

    #[allow(dead_code)]
    fn _tier_test_types() {
        let _a: SideEffectClass = SideEffectClass::Read;
        let _b: TaintFlag = TaintFlag::PageDerived;
        let _c: DataFlowSummary = DataFlowSummary::default();
        let _d: TaskAuthority = TaskAuthority::default();
    }

    #[allow(dead_code)]
    type SideEffectClass = crate::core::policy::classes::SideEffectClass;
    #[allow(dead_code)]
    type TaintFlag = crate::core::policy::taint::TaintFlag;
    #[allow(dead_code)]
    type DataFlowSummary = crate::core::policy::provenance::DataFlowSummary;
    #[allow(dead_code)]
    type TaskAuthority = crate::core::orchestrator::task_authority::TaskAuthority;
    #[allow(dead_code)]
    type TaskConstraints = crate::core::orchestrator::task_authority::TaskConstraints;
    #[allow(dead_code)]
    type DataFlowClass = crate::core::policy::provenance::DataFlowClass;

    impl Default for SideEffectClass {
        fn default() -> Self { Self::Read }
    }
    impl Default for TaintFlag {
        fn default() -> Self { Self::None }
    }
    impl Default for DataFlowSummary {
        fn default() -> Self {
            Self {
                source: String::new(),
                sink: String::new(),
                flow_class: DataFlowClass::TrustedRuntime,
                verdict: None,
            }
        }
    }
    impl Default for TaskAuthority {
        fn default() -> Self {
            Self {
                task_id: String::new(),
                bounded_origins: vec![],
                allowed_actions: vec![],
                constraints: TaskConstraints::default(),
            }
        }
    }
    impl Default for TaskConstraints {
        fn default() -> Self {
            Self {
                max_steps: None,
                max_llm_calls: None,
                max_task_duration_ms: None,
                max_confirmations: None,
                allowed_network_destination_classes: vec![],
                high_stakes_threshold_amount: None,
            }
        }
    }
    impl Default for DataFlowClass {
        fn default() -> Self { Self::TrustedRuntime }
    }

    use crate::core::action::authz::{authorize_action_commitment, verify_action_authorization_capability};


    fn now_monotonic() -> u64 {
        1_000_000
    }

    fn destination() -> String {
        "merchant@payee".into()
    }

    #[test]
    fn authorization_is_bound_to_commitment() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            destination(),
            7,
            300_000,
            now_monotonic(),
        );
        let cap = authorize_action_commitment(&commitment, OperationType::ConsequentialAction);
        assert!(cap.authorizes_commitment(&commitment));
    }

    #[test]
    fn authorization_does_not_authorize_different_commitment() {
        let base = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            destination(),
            7,
            300_000,
            now_monotonic(),
        );
        let cap = authorize_action_commitment(&base, OperationType::ConsequentialAction);
        let changed = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "other@merchant".into(),
            7,
            300_000,
            now_monotonic(),
        );
        assert!(!cap.authorizes_commitment(&changed));
    }

    #[test]
    fn authorization_invalidated_by_epoch_change() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            destination(),
            7,
            300_000,
            now_monotonic(),
        );
        let cap = authorize_action_commitment(&commitment, OperationType::ConsequentialAction);
        let changed_epoch = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            destination(),
            8,
            300_000,
            now_monotonic(),
        );
        assert!(!cap.authorizes_commitment(&changed_epoch));
    }

    #[test]
    fn verification_uses_capability_binding() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            destination(),
            7,
            300_000,
            now_monotonic(),
        );
        let cap = authorize_action_commitment(&commitment, OperationType::ConsequentialAction);
        assert!(verify_action_authorization_capability(
            &cap,
            &commitment,
            &cap.authorization_nonce,
            now_monotonic() + 100
        ));
    }

    #[test]
    fn wrong_nonce_fails_verification() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            destination(),
            7,
            300_000,
            now_monotonic(),
        );
        let cap = authorize_action_commitment(&commitment, OperationType::ConsequentialAction);
        let wrong_nonce: [u8; 32] = rand::random();
        assert!(!verify_action_authorization_capability(
            &cap,
            &commitment,
            &wrong_nonce,
            now_monotonic()
        ));
    }

    #[test]
    fn expiry_invalidates_verification() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            destination(),
            7,
            100,
            now_monotonic(),
        );
        let cap = authorize_action_commitment(&commitment, OperationType::ConsequentialAction);
        let later = cap.expires_at_monotonic + 1;
        assert!(!verify_action_authorization_capability(
            &cap,
            &commitment,
            &cap.authorization_nonce,
            later
        ));
    }

    #[test]
    fn verify_action_authorization_capability_is_reachable_from_policy_layer() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            destination(),
            7,
            300_000,
            now_monotonic(),
        );
        let nonce: [u8; 32] = rand::random();
        let cap = ActionAuthorizationCapability::new(
            &commitment,
            OperationType::ConsequentialAction,
            nonce,
            now_monotonic(),
            300_000,
        );
        assert!(verify_action_authorization_capability(
            &cap,
            &commitment,
            &nonce,
            now_monotonic()
        ));
    }
}
