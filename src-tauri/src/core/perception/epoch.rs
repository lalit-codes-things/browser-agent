// Perception epoch.
//
// C-141: model-proposed actions are bound to the state epoch they were
//        derived from; stale actions are rejected when relevant state
//        changes before execution.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PerceptionEpoch {
    pub value: u64,
    pub frame_id: String,
    pub loader_id: String,
}

impl PerceptionEpoch {
    pub fn new(value: u64, frame_id: String, loader_id: String) -> Self {
        Self { value, frame_id, loader_id }
    }

    pub fn matches_loader(&self, loader_id: &str) -> bool {
        self.loader_id == loader_id
    }

    pub fn matches_frame(&self, frame_id: &str) -> bool {
        self.frame_id == frame_id
    }
}

pub fn stale_after(epoch: PerceptionEpoch, current: PerceptionEpoch) -> bool {
    // A simplified staleness heuristic for the Phase 1 slice.
    // Phase 3 completes the full target/frame/loader identity rules.
    epoch.value != current.value
        || epoch.frame_id != current.frame_id
        || epoch.loader_id != current.loader_id
}
