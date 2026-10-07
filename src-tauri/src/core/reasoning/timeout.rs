// Inference timeout.
//
// C-146: every inference call has a per-call wall-clock timeout with
//        abort callbacks wired through the runtime; task abort cancels
//        in-flight generation.
//
// We use monotonic time for budgets (C-61).

use std::sync::{Arc, Mutex};
use crate::core::orchestrator::budgets::BudgetPolicy;

pub struct InferenceTimeout {
    pub deadline: Option<std::time::Instant>,
    pub budget: Arc<Mutex<BudgetPolicy>>,
}

impl InferenceTimeout {
    pub fn new(budget: Arc<Mutex<BudgetPolicy>>, budget_policy: BudgetPolicy) -> Self {
        let deadline = budget_policy
            .max_task_duration_ms
            .map(|ms| std::time::Instant::now() + std::time::Duration::from_millis(ms));
        Self { deadline, budget }
    }

    pub fn remaining_budget_ms(&self) -> Option<u64> {
        self.deadline
            .map(|d| {
                d.duration_since(std::time::Instant::now())
                    .as_millis()
                    .try_into()
                    .unwrap_or(0)
            })
            .filter(|&x| x > 0)
    }

    pub fn is_expired(&self) -> bool {
        match self.deadline {
            Some(d) => std::time::Instant::now() >= d,
            None => false,
        }
    }
}
