// Clocks.
//
// C-61: budgets use monotonic clocks.

pub struct MonotonicClock;

impl MonotonicClock {
    pub fn now_nanos() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }
}
