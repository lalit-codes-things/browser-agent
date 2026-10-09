// Monotonic process clock for in-memory budgets and expiry.
// Persisted records must use a wall-clock timestamp separately.

use std::sync::OnceLock;
use std::time::Instant;

static PROCESS_START: OnceLock<Instant> = OnceLock::new();

pub struct MonotonicClock;

impl MonotonicClock {
    pub fn now_nanos() -> u64 {
        PROCESS_START
            .get_or_init(Instant::now)
            .elapsed()
            .as_nanos()
            .min(u128::from(u64::MAX)) as u64
    }
}
