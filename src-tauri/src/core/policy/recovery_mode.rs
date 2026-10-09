// Recovery mode policy.
//
// C-35: recovery mode uses tighter budgets and is protected against
//        attacker-triggered uncertainty with rate limiting.

use crate::core::orchestrator::budgets::BudgetPolicy;

pub struct RecoveryBudgetPolicy {
    pub base: BudgetPolicy,
    pub tightened: bool,
}

impl RecoveryBudgetPolicy {
    pub fn new(base: BudgetPolicy) -> Self {
        Self {
            base,
            tightened: true,
        }
    }
}
