// Semantic references as stable abstractions.
//
// C-85: semantic references are stable abstractions over targets, not
//        brittle DOM selectors. A reference resolves against a specific
//        perception epoch; cross-epoch resolution requires re-perception.
//
// References are opaque to the model: the model echoes them back; the
// runtime resolves them against the graph it derived them from.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SemanticReference {
    /// Opaque runtime-assigned token (e.g. "node-41"). Not a selector.
    pub token: String,
    /// Epoch value the reference was derived from.
    pub epoch: u64,
    pub frame_id: String,
    pub loader_id: String,
}

impl SemanticReference {
    pub fn new(token: String, epoch: u64, frame_id: String, loader_id: String) -> Self {
        Self { token, epoch, frame_id, loader_id }
    }

    /// A reference is only valid against the epoch it was derived from.
    /// Cross-epoch resolution is rejected: the caller re-perceives.
    pub fn valid_against(&self, epoch: u64, frame_id: &str, loader_id: &str) -> bool {
        self.epoch == epoch && self.frame_id == frame_id && self.loader_id == loader_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// Reference from a different epoch/frame/loader.
    Stale,
    /// Reference not present in the current graph.
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(epoch: u64) -> SemanticReference {
        SemanticReference::new("node-41".into(), epoch, "F1".into(), "L1".into())
    }

    #[test]
    fn valid_within_same_epoch() {
        assert!(reference(7).valid_against(7, "F1", "L1"));
    }

    #[test]
    fn stale_across_epoch() {
        assert!(!reference(7).valid_against(8, "F1", "L1"));
    }

    #[test]
    fn stale_across_frame() {
        assert!(!reference(7).valid_against(7, "F2", "L1"));
    }

    #[test]
    fn stale_across_loader() {
        assert!(!reference(7).valid_against(7, "F1", "L2"));
    }
}
