// Mutation watcher.
//
// C-81: mutation-noise suppression; C-65: Phase-1 controlled mutations.
// C-141: relevant mutations invalidate epochs. Noise must not.
//
// The watcher subscribes to runtime mutation events (injected via CDP),
// classifies each observation through noise.rs, and advances epochs only
// for relevant/uncertain classes.

use std::time::{Duration, Instant};

use crate::core::perception::noise::{classify, MutationClass, MutationObservation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Epoch {
    pub value: u64,
}

#[derive(Debug)]
pub struct MutationWatcher {
    current: Epoch,
    stats: WatcherStats,
}

#[derive(Debug, Default, Clone)]
pub struct WatcherStats {
    pub relevant: u64,
    pub noise: u64,
    pub uncertain: u64,
}

impl MutationWatcher {
    pub fn new() -> Self {
        Self {
            current: Epoch { value: 0 },
            stats: WatcherStats::default(),
        }
    }

    pub fn current_epoch(&self) -> Epoch {
        self.current
    }

    pub fn stats(&self) -> &WatcherStats {
        &self.stats
    }

    /// Observe and classify a mutation. Returns the (possibly advanced)
    /// epoch. Noise leaves the epoch unchanged.
    pub fn observe(&mut self, observation: &MutationObservation, _now: Instant) -> Epoch {
        match classify(observation) {
            MutationClass::Noise => {
                self.stats.noise += 1;
            }
            MutationClass::Relevant => {
                self.stats.relevant += 1;
                self.current.value += 1;
            }
            MutationClass::Uncertain => {
                // Fail-closed: uncertain mutations invalidate the epoch.
                self.stats.uncertain += 1;
                self.current.value += 1;
            }
        }
        self.current
    }

    /// Convenience for testing: advance with a controlled relevant mutation.
    pub fn advance(&mut self, now: Instant) -> Epoch {
        self.observe(
            &MutationObservation {
                frame_id: "test".into(),
                role: None,
                text_changed: true,
                bounds_changed: false,
                node_added: false,
                node_removed: false,
                cosmetic_attribute_only: false,
            },
            now,
        )
    }
}

impl Default for MutationWatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal pacing guard for observation loops: batches must not be
/// processed faster than `min_interval`, keeping mutation storms from
/// starving the runtime. Bounds are runtime config, not catalog values.
#[derive(Debug, Clone, Copy)]
pub struct WatcherPacing {
    pub min_interval: Duration,
    last: Option<Instant>,
}

impl WatcherPacing {
    pub fn new(min_interval: Duration) -> Self {
        Self {
            min_interval,
            last: None,
        }
    }

    pub fn ready(&mut self, now: Instant) -> bool {
        match self.last {
            Some(t) if now.duration_since(t) < self.min_interval => false,
            _ => {
                self.last = Some(now);
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn obs_relevant() -> MutationObservation {
        MutationObservation {
            frame_id: "F1".into(),
            role: Some("button".into()),
            text_changed: true,
            bounds_changed: false,
            node_added: false,
            node_removed: false,
            cosmetic_attribute_only: false,
        }
    }

    fn obs_noise() -> MutationObservation {
        MutationObservation {
            frame_id: "F1".into(),
            role: None,
            text_changed: false,
            bounds_changed: false,
            node_added: false,
            node_removed: false,
            cosmetic_attribute_only: true,
        }
    }

    #[test]
    fn noise_does_not_advance_epoch() {
        let t0 = Instant::now();
        let mut w = MutationWatcher::new();
        let before = w.current_epoch();
        w.observe(&obs_noise(), t0);
        assert_eq!(w.current_epoch(), before);
        assert_eq!(w.stats().noise, 1);
    }

    #[test]
    fn relevant_advances_epoch() {
        let t0 = Instant::now();
        let mut w = MutationWatcher::new();
        let e1 = w.observe(&obs_relevant(), t0);
        let e2 = w.observe(&obs_relevant(), t0 + Duration::from_millis(10));
        assert_ne!(e1, e2);
        assert_eq!(w.stats().relevant, 2);
    }

    #[test]
    fn uncertain_advances_epoch_fail_closed() {
        let t0 = Instant::now();
        let mut w = MutationWatcher::new();
        let unclassified = MutationObservation {
            frame_id: "F1".into(),
            role: None,
            text_changed: false,
            bounds_changed: false,
            node_added: false,
            node_removed: false,
            cosmetic_attribute_only: false,
        };
        let before = w.current_epoch();
        w.observe(&unclassified, t0);
        assert_ne!(
            w.current_epoch(),
            before,
            "uncertain mutation must invalidate epoch"
        );
    }

    #[test]
    fn pacing_gate() {
        let t0 = Instant::now();
        let mut p = WatcherPacing::new(Duration::from_millis(50));
        assert!(p.ready(t0));
        assert!(!p.ready(t0 + Duration::from_millis(20)));
        assert!(p.ready(t0 + Duration::from_millis(60)));
    }
}
