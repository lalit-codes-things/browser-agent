// Semantic stabilization.
//
// C-59: perception waits for semantic stabilization before action;
//        livelock detection uses the locked detector in livelock.rs.
//
// Stabilization is defined as: no Relevant mutations and no epoch change
// for `quiet_period` within the observation window. Quiet-period bounds
// are TUNABLE/undefined in Revision 2 — supplied by the caller, never
// invented here.

use std::time::{Duration, Instant};

use crate::core::perception::noise::MutationClass;

#[derive(Debug, Clone, Copy)]
pub struct StabilizationPolicy {
    /// Required quiet period. Undefined in catalog; caller supplies.
    pub quiet_period: Duration,
    /// Observation window upper bound. Undefined in catalog; caller supplies.
    pub max_wait: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stabilization {
    Stable,
    Waiting,
    /// Livelock detector fired; caller must stop or re-perceive via
    /// recovery mode. Not a silent retry.
    Livelocked,
}

pub struct StabilizationWatcher {
    policy: StabilizationPolicy,
    last_relevant_change: Option<Instant>,
    observation_started: Instant,
}

impl StabilizationWatcher {
    pub fn new(policy: StabilizationPolicy, now: Instant) -> Self {
        Self {
            policy,
            last_relevant_change: Some(now),
            observation_started: now,
        }
    }

    pub fn record_mutation(&mut self, class: MutationClass, now: Instant) {
        if class != MutationClass::Noise {
            self.last_relevant_change = Some(now);
        }
    }

    pub fn status(&self, now: Instant) -> Stabilization {
        if now.duration_since(self.observation_started) >= self.policy.max_wait {
            // Window exhausted without stability: report Waiting truthfully;
            // the livelock monitor decides whether this is livelock.
            return Stabilization::Waiting;
        }
        match self.last_relevant_change {
            Some(t) if now.duration_since(t) >= self.policy.quiet_period => Stabilization::Stable,
            _ => Stabilization::Waiting,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn policy() -> StabilizationPolicy {
        StabilizationPolicy {
            quiet_period: Duration::from_millis(300),
            max_wait: Duration::from_secs(5),
        }
    }

    #[test]
    fn unstable_immediately_after_change() {
        let t0 = Instant::now();
        let mut w = StabilizationWatcher::new(policy(), t0);
        w.record_mutation(MutationClass::Relevant, t0 + Duration::from_millis(50));
        assert_eq!(w.status(t0 + Duration::from_millis(100)), Stabilization::Waiting);
    }

    #[test]
    fn stable_after_quiet_period() {
        let t0 = Instant::now();
        let mut w = StabilizationWatcher::new(policy(), t0);
        w.record_mutation(MutationClass::Relevant, t0);
        assert_eq!(w.status(t0 + Duration::from_millis(400)), Stabilization::Stable);
    }

    #[test]
    fn noise_does_not_reset_quiet_clock() {
        let t0 = Instant::now();
        let mut w = StabilizationWatcher::new(policy(), t0);
        w.record_mutation(MutationClass::Relevant, t0);
        w.record_mutation(MutationClass::Noise, t0 + Duration::from_millis(100));
        assert_eq!(w.status(t0 + Duration::from_millis(400)), Stabilization::Stable);
    }

    #[test]
    fn relevant_mutation_resets_quiet_clock() {
        let t0 = Instant::now();
        let mut w = StabilizationWatcher::new(policy(), t0);
        w.record_mutation(MutationClass::Relevant, t0);
        w.record_mutation(MutationClass::Uncertain, t0 + Duration::from_millis(350));
        assert_eq!(w.status(t0 + Duration::from_millis(400)), Stabilization::Waiting);
    }
}
