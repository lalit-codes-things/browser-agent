// Budgets.
//
// C-149: hard budgets for MAX_STEPS, MAX_LLM_CALLS, MAX_RECOVERY_ATTEMPTS,
// MAX_TASK_DURATION, MAX_CONFIRMATIONS, graph size, screenshot size/count,
// response/streaming size, download size, safe unified-memory pressure.
//
// Exceeding any triggers C-63 stop semantics.
//
// Numeric values are undefined in Revision 2. Do not invent them here.

use std::time::Instant;
use crate::security::clocks::MonotonicClock;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BudgetSnapshot {
    pub steps_used: u32,
    pub llm_calls_used: u32,
    pub recovery_attempts_used: u32,
    pub confirmations_used: u32,
    pub wall_duration_ms: u64,
}

pub struct BudgetPolicy {
    pub max_steps: Option<u32>,
    pub max_llm_calls: Option<u32>,
    pub max_recovery_attempts: Option<u32>,
    pub max_task_duration_ms: Option<u64>,
    pub max_confirmations: Option<u32>,
    pub clock: MonotonicClock,
    pub started_at: Instant,
}

impl BudgetPolicy {
    pub fn new(max: crate::config::RuntimeConfig) -> Self {
        Self {
            max_steps: max.max_steps,
            max_llm_calls: max.max_llm_calls,
            max_recovery_attempts: max.max_recovery_attempts,
            max_task_duration_ms: max.max_task_duration_ms,
            max_confirmations: max.max_confirmations,
            clock: MonotonicClock,
            started_at: Instant::now(),
        }
    }

    pub fn snapshot(&self, steps: u32, llm_calls: u32, recovery_attempts: u32, confirmations: u32) -> BudgetSnapshot {
        BudgetSnapshot {
            steps_used: steps,
            llm_calls_used: llm_calls,
            recovery_attempts_used: recovery_attempts,
            confirmations_used: confirmations,
            wall_duration_ms: self
                .started_at
                .elapsed()
                .as_millis()
                .try_into()
                .unwrap_or(u64::MAX),
        }
    }

    pub fn any_exhausted(&self, snapshot: &BudgetSnapshot) -> Option<String> {
        if let Some(max) = self.max_steps {
            if snapshot.steps_used >= max {
                return Some("MAX_STEPS".into());
            }
        }
        if let Some(max) = self.max_llm_calls {
            if snapshot.llm_calls_used >= max {
                return Some("MAX_LLM_CALLS".into());
            }
        }
        if let Some(max) = self.max_recovery_attempts {
            if snapshot.recovery_attempts_used >= max {
                return Some("MAX_RECOVERY_ATTEMPTS".into());
            }
        }
        if let Some(max) = self.max_task_duration_ms {
            if snapshot.wall_duration_ms >= max {
                return Some("MAX_TASK_DURATION".into());
            }
        }
        if let Some(max) = self.max_confirmations {
            if snapshot.confirmations_used >= max {
                return Some("MAX_CONFIRMATIONS".into());
            }
        }
        None
    }
}
