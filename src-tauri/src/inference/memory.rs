// Memory / residency policy.
//
// C-147: model-residency policy handles external memory pressure: pressure-
//        keyed unload, task parking, budgeted reload, jetsam avoidance.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPressure {
    Normal,
    Elevated,
    Critical,
}

pub struct MemoryPolicy;

impl MemoryPolicy {
    pub fn evaluate_pressure(_kv_cache_bytes: u64, _system_available_bytes: u64) -> MemoryPressure {
        if _system_available_bytes < _kv_cache_bytes {
            MemoryPressure::Critical
        } else if _system_available_bytes < 2 * _kv_cache_bytes {
            MemoryPressure::Elevated
        } else {
            MemoryPressure::Normal
        }
    }
}
