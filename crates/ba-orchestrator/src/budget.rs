//! Task budgets (BA-TASK-BUDGET-001, BA-RESOURCE-001).
//!
//! Limits are hard ceilings for the measured task envelope, not safety
//! authorization. Only one extension of up to 50% per exhausted dimension is
//! allowed per task, and it never overrides a policy refusal, a security hard
//! stop, the no-autonomous-irreversible-action rule or a hard memory ceiling.

use ba_domain::{BudgetField, TaskBudgetView};

/// A budget dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BudgetDimension {
    /// Policy-evaluated action proposals.
    ActionProposals,
    /// Dispatched actions.
    DispatchedActions,
    /// Agent-initiated top-level navigations.
    Navigations,
    /// Cumulative model input tokens.
    InputTokens,
    /// Cumulative model output tokens.
    OutputTokens,
    /// Active task execution time in milliseconds.
    ActiveTime,
}

impl BudgetDimension {
    /// Returns the plain-language name.
    #[must_use]
    pub const fn plain_language(self) -> &'static str {
        match self {
            Self::ActionProposals => "actions reviewed",
            Self::DispatchedActions => "actions performed",
            Self::Navigations => "page openings the agent started",
            Self::InputTokens => "model input tokens",
            Self::OutputTokens => "model output tokens",
            Self::ActiveTime => "active task time",
        }
    }
}

/// Why a budget could not be extended.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BudgetError {
    /// The task already used its single extension.
    #[error("this task has already used its one budget extension")]
    AlreadyExtended,
    /// No dimension has reached its ceiling.
    #[error("no budget ceiling has been reached, so there is nothing to extend")]
    NotExhausted,
}

/// The hard per-task budget for the reference profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskBudget {
    /// Policy-evaluated action proposals.
    pub action_proposals: BudgetField,
    /// Dispatched actions.
    pub dispatched_actions: BudgetField,
    /// Agent-initiated top-level navigations.
    pub navigations: BudgetField,
    /// Cumulative model input tokens.
    pub input_tokens: BudgetField,
    /// Cumulative model output tokens.
    pub output_tokens: BudgetField,
    /// Active execution time in milliseconds.
    pub active_ms: BudgetField,
    /// Waiting and paused wall time, shown separately and never a ceiling.
    pub waiting_ms: u64,
    /// How many extensions were approved for this task.
    pub extensions_used: u8,
}

impl TaskBudget {
    /// Defaults for the 8 GB reference profile.
    #[must_use]
    pub const fn reference_profile() -> Self {
        Self {
            action_proposals: BudgetField { limit: 60, used: 0 },
            dispatched_actions: BudgetField { limit: 50, used: 0 },
            navigations: BudgetField { limit: 20, used: 0 },
            input_tokens: BudgetField {
                limit: 64_000,
                used: 0,
            },
            output_tokens: BudgetField {
                limit: 8_000,
                used: 0,
            },
            active_ms: BudgetField {
                limit: 15 * 60 * 1_000,
                used: 0,
            },
            waiting_ms: 0,
            extensions_used: 0,
        }
    }

    /// Returns the task budget as a renderable view.
    #[must_use]
    pub const fn view(&self) -> TaskBudgetView {
        TaskBudgetView {
            action_proposals: self.action_proposals,
            dispatched_actions: self.dispatched_actions,
            navigations: self.navigations,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            active_ms: self.active_ms,
            waiting_ms: self.waiting_ms,
            extensions_used: self.extensions_used,
        }
    }

    /// Returns the field for `dimension`.
    #[must_use]
    pub const fn field(&self, dimension: BudgetDimension) -> BudgetField {
        match dimension {
            BudgetDimension::ActionProposals => self.action_proposals,
            BudgetDimension::DispatchedActions => self.dispatched_actions,
            BudgetDimension::Navigations => self.navigations,
            BudgetDimension::InputTokens => self.input_tokens,
            BudgetDimension::OutputTokens => self.output_tokens,
            BudgetDimension::ActiveTime => self.active_ms,
        }
    }

    fn field_mut(&mut self, dimension: BudgetDimension) -> &mut BudgetField {
        match dimension {
            BudgetDimension::ActionProposals => &mut self.action_proposals,
            BudgetDimension::DispatchedActions => &mut self.dispatched_actions,
            BudgetDimension::Navigations => &mut self.navigations,
            BudgetDimension::InputTokens => &mut self.input_tokens,
            BudgetDimension::OutputTokens => &mut self.output_tokens,
            BudgetDimension::ActiveTime => &mut self.active_ms,
        }
    }

    /// Returns every dimension, in a stable order.
    #[must_use]
    pub const fn dimensions() -> [BudgetDimension; 6] {
        [
            BudgetDimension::ActionProposals,
            BudgetDimension::DispatchedActions,
            BudgetDimension::Navigations,
            BudgetDimension::InputTokens,
            BudgetDimension::OutputTokens,
            BudgetDimension::ActiveTime,
        ]
    }

    /// Returns the dimensions that have reached their ceiling.
    #[must_use]
    pub fn exhausted(&self) -> Vec<BudgetDimension> {
        Self::dimensions()
            .into_iter()
            .filter(|dimension| self.field(*dimension).exhausted())
            .collect()
    }

    /// Consumes one unit of `dimension`.
    ///
    /// Returns `false` when the ceiling is already reached; the caller must not
    /// treat that as permission to continue.
    pub fn consume(&mut self, dimension: BudgetDimension, amount: u64) -> bool {
        self.field_mut(dimension).try_consume(amount)
    }

    /// Records active execution time.
    pub fn accrue_active(&mut self, ms: u64) {
        self.active_ms.used = self.active_ms.used.saturating_add(ms);
    }

    /// Records waiting or paused wall time.
    pub fn accrue_waiting(&mut self, ms: u64) {
        self.waiting_ms = self.waiting_ms.saturating_add(ms);
    }

    /// Raises every exhausted ceiling by up to 50%, once per task.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError`] when the extension was already used or nothing
    /// is exhausted. The new ceilings are returned so the UI can show the exact
    /// figures before approval.
    pub fn extend(
        &mut self,
        dimensions: &[BudgetDimension],
    ) -> Result<Vec<(BudgetDimension, u64)>, BudgetError> {
        if self.extensions_used >= 1 {
            return Err(BudgetError::AlreadyExtended);
        }
        if dimensions.is_empty() {
            return Err(BudgetError::NotExhausted);
        }
        let mut raised = Vec::new();
        for dimension in dimensions {
            let field = self.field_mut(*dimension);
            let increment = field.limit / 2;
            field.limit = field.limit.saturating_add(increment);
            raised.push((*dimension, field.limit));
        }
        self.extensions_used = self.extensions_used.saturating_add(1);
        Ok(raised)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_defaults_match_the_specification() {
        let budget = TaskBudget::reference_profile();
        assert_eq!(budget.action_proposals.limit, 60);
        assert_eq!(budget.dispatched_actions.limit, 50);
        assert_eq!(budget.navigations.limit, 20);
        assert_eq!(budget.input_tokens.limit, 64_000);
        assert_eq!(budget.output_tokens.limit, 8_000);
        assert_eq!(budget.active_ms.limit, 900_000);
        assert_eq!(budget.extensions_used, 0);
    }

    #[test]
    fn consumption_stops_at_the_ceiling() {
        let mut budget = TaskBudget::reference_profile();
        for _ in 0..60 {
            assert!(budget.consume(BudgetDimension::ActionProposals, 1));
        }
        assert!(!budget.consume(BudgetDimension::ActionProposals, 1));
        assert_eq!(budget.exhausted(), vec![BudgetDimension::ActionProposals]);
    }

    #[test]
    fn extension_raises_only_exhausted_ceilings_and_only_once() {
        let mut budget = TaskBudget::reference_profile();
        for _ in 0..20 {
            budget.consume(BudgetDimension::Navigations, 1);
        }
        let raised = budget
            .extend(&[BudgetDimension::Navigations])
            .expect("one extension is allowed");
        assert_eq!(raised, vec![(BudgetDimension::Navigations, 30)]);
        assert_eq!(budget.navigations.limit, 30);
        assert_eq!(budget.action_proposals.limit, 60);
        assert!(matches!(
            budget.extend(&[BudgetDimension::Navigations]),
            Err(BudgetError::AlreadyExtended)
        ));
    }

    #[test]
    fn extending_nothing_is_refused() {
        let mut budget = TaskBudget::reference_profile();
        assert!(matches!(budget.extend(&[]), Err(BudgetError::NotExhausted)));
        assert_eq!(budget.extensions_used, 0);
    }

    #[test]
    fn waiting_time_is_tracked_separately() {
        let mut budget = TaskBudget::reference_profile();
        budget.accrue_active(1_000);
        budget.accrue_waiting(5_000);
        let view = budget.view();
        assert_eq!(view.active_ms.used, 1_000);
        assert_eq!(view.waiting_ms, 5_000);
        assert!(!view.active_ms.exhausted());
    }
}
