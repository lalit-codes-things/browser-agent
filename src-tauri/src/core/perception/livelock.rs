// Livelock detector.
//
// C-59: semantic stabilization uses the locked livelock detector, with
//        defaults of 8 epoch changes within 2 seconds and 3 failed windows
//        within 10 seconds.
//
// Quiet-window min/max are not specified in Revision 2 (TUNABLE, undefined).
// We do not invent them here.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct LivelockConfig {
    pub max_epoch_changes_in_window: u32,
    pub epoch_window_duration: Duration,
    pub max_failed_windows_in_window: u32,
    pub failed_window_duration: Duration,
}

impl Default for LivelockConfig {
    fn default() -> Self {
        Self {
            max_epoch_changes_in_window: 8,
            epoch_window_duration: Duration::from_secs(2),
            max_failed_windows_in_window: 3,
            failed_window_duration: Duration::from_secs(10),
        }
    }
}

pub struct LivelockMonitor {
    config: LivelockConfig,
    epoch_changes: VecDeque<Instant>,
    failed_windows: VecDeque<Instant>,
}

impl LivelockMonitor {
    pub fn new(config: LivelockConfig) -> Self {
        Self {
            config,
            epoch_changes: VecDeque::new(),
            failed_windows: VecDeque::new(),
        }
    }

    pub fn record_epoch_change(&mut self, now: Instant) {
        self.epoch_changes.push_back(now);
        self.prune_epoch_changes(now);
    }

    pub fn record_failed_window(&mut self, now: Instant) {
        self.failed_windows.push_back(now);
        self.prune_failed_windows(now);
    }

    fn prune_epoch_changes(&mut self, now: Instant) {
        while let Some(t) = self.epoch_changes.front() {
            if now.duration_since(*t) > self.config.epoch_window_duration {
                self.epoch_changes.pop_front();
            } else {
                break;
            }
        }
    }

    fn prune_failed_windows(&mut self, now: Instant) {
        while let Some(t) = self.failed_windows.front() {
            if now.duration_since(*t) > self.config.failed_window_duration {
                self.failed_windows.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn is_livelocked(&self, now: Instant) -> bool {
        self.epoch_changes.len() > self.config.max_epoch_changes_in_window as usize
            || self.failed_windows.len() > self.config.max_failed_windows_in_window as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn livelock_after_many_epoch_changes() {
        let mut monitor = LivelockMonitor::new(LivelockConfig::default());
        let base = Instant::now();
        for i in 0..=8 {
            monitor.record_epoch_change(base + Duration::from_millis(i as u64));
        }
        assert!(monitor.is_livelocked(base + Duration::from_secs(1)));
    }
}
