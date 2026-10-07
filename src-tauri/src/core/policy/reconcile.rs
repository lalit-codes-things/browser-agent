// Task-authority reconciliation.
//
// C-33: task-authority parameters must reconcile with action parameters:
//        exact recipient/account/destination match, locale-aware bounded
//        amount deltas, and item identity match; divergence is STOP.
// C-32: navigation URLs are treated as potential exfiltration channels.
//
// Reconciliation is backend-owned and auditable.

use crate::core::orchestrator::task_authority::TaskAuthority;
use crate::core::policy::classes::SideEffectClass;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReconciliationResult {
    pub reconciled: bool,
    pub reason: String,
}

pub struct Reconciliation;

impl Reconciliation {
    pub fn check(_authority: &TaskAuthority, _action_class: SideEffectClass, _destination: &str, _amount: Option<&str>) -> ReconciliationResult {
        // Placeholder: real reconciliation is audited.
        ReconciliationResult {
            reconciled: false,
            reason: "Reconciliation is scheduled".into(),
        }
    }
}
