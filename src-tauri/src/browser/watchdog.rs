// Watchdog.
//
// C-78: jetsam-aware safe-state preservation distinguishes watchdog stops
//        from SIGKILL; renderer/memory watchdogs, caps, and safe termination
//        are required.

pub struct Watchdog;

impl Watchdog {
    pub fn start(_memory_cap_bytes: Option<u64>) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented(
            "Watchdog::start is scheduled".into(),
        ))
    }

    pub fn preserve_safe_state(_reason: &str) -> Result<String, crate::Error> {
        Err(crate::Error::NotImplemented(
            "Watchdog::preserve_safe_state is scheduled".into(),
        ))
    }
}
