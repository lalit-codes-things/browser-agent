// Authorization tiers.
//
// C-07: authorization tier is a separate dimension: NONE / POLICY_ONLY /
//        NATIVE_CONFIRM / BIOMETRIC_CONFIRM.
// C-08: tier derived by Policy using side-effect class + taint, destination,
//        data flow, amount thresholds, task authority.
// C-140: tier under-classification is the measured, gated, adversarially
//         tested safety property.
//
// The tier dimension is separate from side-effect class (C-07).

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

    /// Fail upward: uncertainty in tier derivation must not downgrade.
    pub fn from_opt_tier(opt: Option<Self>) -> Self {
        opt.unwrap_or(Self::NativeConfirm)
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
    /// C-08: derive tier from trusted runtime data.
    ///
    /// This is a placeholder implementation that enforces the tier
    /// dimension and fails upward. The numeric HIGH_STAKES purchase
    /// threshold is undefined in Revision 2 (C-08, C-15), so we do not
    /// hardcode it here.
    pub fn derive(_input: TierDerivationInput) -> Result<AuthorizationTier, crate::Error> {
        // Placeholder: real derivation is Policy-owned and audited.
        Err(crate::Error::NotImplemented(
            "TierDerivation::derive is scheduled; HIGH_STAKES threshold undefined".into(),
        ))
    }
}
