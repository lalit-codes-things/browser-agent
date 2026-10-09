// Confirmation policy.
//
// C-14: confirmation is tiered.
// C-15: fresh Touch ID only for HIGH_STAKES payments/financial actions and
//        important sensitive decisions.
// C-16: ordinary login/browsing/typing/credential fill/reversible/routine
//        execution do not use biometrics.
// C-17: other consequential actions use native confirmation and policy
//        controls without biometrics.
// C-22: model knows only REQUEST_CONFIRMATION; it does not know or control
//        the biometric mechanism.

use crate::core::policy::tiers::AuthorizationTier;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfirmationRequirement {
    pub required: bool,
    pub tier: AuthorizationTier,
    pub reason: Option<String>,
}

pub struct ConfirmationPolicy;

impl ConfirmationPolicy {
    pub fn requirement(
        tier: AuthorizationTier,
        action_class: crate::core::policy::classes::SideEffectClass,
    ) -> ConfirmationRequirement {
        let required = !matches!(
            (tier, action_class.effective_class()),
            (
                AuthorizationTier::None,
                crate::core::policy::classes::SideEffectClass::Read
            ) | (
                AuthorizationTier::PolicyOnly,
                crate::core::policy::classes::SideEffectClass::Read
            )
        );
        ConfirmationRequirement {
            required,
            tier,
            reason: Some(if required {
                "Trusted confirmation is required".into()
            } else {
                "Read-only policy action".into()
            }),
        }
    }
}
