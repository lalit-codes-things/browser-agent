// Policy engine.
//
// C-08: tier derived by Policy, never by the model.
// C-20: no silent written-policy override that disables HIGH_STAKES auth.
// C-21: CI/headless/replay cannot perform real HIGH_STAKES actions outside
//        isolated synthetic policy.
//
// The engine coordinates classes, tiers, taint, scopes, provenance,
// declassification, reconciliation, locale amounts, recovery mode, rate
// limiting, and confirmation. Submodule files live alongside this file
// in core/policy/ (declared in mod.rs); engine.rs re-exports them.

pub use super::classes::SideEffectClass;
pub use super::tiers::{AuthorizationTier, TierDerivation, TierDerivationInput};
pub use super::scopes::CredentialScope;
pub use super::taint::TaintFlag;
pub use super::provenance::{DataFlowClass, DataFlowSummary};
pub use super::declassify::DeclassificationPolicy;
pub use super::reconcile::{Reconciliation, ReconciliationResult};
pub use super::locale_amount::AmountDisplay;
pub use super::recovery_mode::RecoveryBudgetPolicy;
pub use super::rate_limit::ConfirmationRateLimiter;
pub use super::confirmation::{ConfirmationPolicy, ConfirmationRequirement};
