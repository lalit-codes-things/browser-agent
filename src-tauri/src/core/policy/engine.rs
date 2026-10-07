// Policy engine.
//
// C-08: tier derived by Policy, never by the model.
// C-20: no silent written-policy override that disables HIGH_STAKES auth.
// C-21: CI/headless/replay cannot perform real HIGH_STAKES actions outside
//        isolated synthetic policy.
//
// The engine coordinates classes, tiers, taint, scopes, provenance,
// declassification, reconciliation, locale amounts, recovery mode, rate
// limiting, and confirmation.

pub mod classes;
pub mod tiers;
pub mod scopes;
pub mod taint;
pub mod provenance;
pub mod declassify;
pub mod reconcile;
pub mod locale_amount;
pub mod recovery_mode;
pub mod rate_limit;
pub mod confirmation;

pub use classes::SideEffectClass;
pub use tiers::{AuthorizationTier, TierDerivation, TierDerivationInput};
pub use scopes::CredentialScope;
pub use taint::TaintFlag;
pub use provenance::{DataFlowClass, DataFlowSummary};
pub use declassify::DeclassificationPolicy;
pub use reconcile::{Reconciliation, ReconciliationResult};
pub use locale_amount::AmountDisplay;
pub use recovery_mode::RecoveryBudgetPolicy;
pub use rate_limit::ConfirmationRateLimiter;
pub use confirmation::{ConfirmationPolicy, ConfirmationRequirement};
