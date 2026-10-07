// Rate limiting for high-stakes confirmation.
//
// C-18: high-stakes biometric prompts rate-limited to 3 rejected attempts
//        per 60 seconds; native-confirmation + master-password fallback
//        symmetrically rate-limited and audited.
//
// This module is the rate-limit boundary. Phase 4 completes the integration
// with actual biometric prompts.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct ConfirmationRateLimiter {
    rejected_attempts: VecDeque<Instant>,
    window: Duration,
    max_rejected: u32,
}

impl ConfirmationRateLimiter {
    pub fn new(max_rejected: u32, window_secs: u64) -> Self {
        Self {
            rejected_attempts: VecDeque::new(),
            window: Duration::from_secs(window_secs),
            max_rejected,
        }
    }

    pub fn record_rejection(&mut self, now: Instant) {
        self.rejected_attempts.push_back(now);
        self.prune(now);
    }

    fn prune(&mut self, now: Instant) {
        while let Some(t) = self.rejected_attempts.front() {
            if now.duration_since(*t) > self.window {
                self.rejected_attempts.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn can_prompt(&self, _now: Instant) -> bool {
        self.rejected_attempts.len() < self.max_rejected as usize
    }

    pub fn rejected_count(&self) -> usize {
        self.rejected_attempts.len()
    }
}
