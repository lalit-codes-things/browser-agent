// Open shadow DOM handling.
//
// C-80: open shadow DOM is supported; closed shadow DOM is not promised
//        and is surfaced as UNKNOWN, never approximated.
//
// Shadow hosts and their subtrees are aggregated into the semantic state
// graph by the perception pipeline; this module owns the boundary types
// and the closed-shadow accounting.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ShadowDomMode {
    /// Open shadow root: contents extracted.
    Open,
    /// Closed shadow root: contents NOT extractable. Explicit UNKNOWN.
    Closed,
    /// Shadow root not yet attached or detached during observation.
    Detached,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShadowHostObservation {
    pub frame_id: String,
    /// Opaque host node identifier from extraction.
    pub host_token: String,
    pub mode: ShadowDomMode,
    /// Number of extractable child nodes (0 for closed/detached).
    pub extractable_children: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShadowExtraction {
    Extracted {
        children: Vec<ShadowHostObservation>,
    },
    /// Closed shadow roots are reported, not guessed.
    NotExtractable { reason: ClosedReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClosedReason {
    ClosedRoot,
    DetachedRoot,
}

pub fn aggregate(hosts: &[ShadowHostObservation]) -> ShadowExtraction {
    let closed: Vec<&ShadowHostObservation> = hosts
        .iter()
        .filter(|h| matches!(h.mode, ShadowDomMode::Closed | ShadowDomMode::Detached))
        .collect();

    if let Some(h) = closed.first() {
        let reason = match h.mode {
            ShadowDomMode::Closed => ClosedReason::ClosedRoot,
            _ => ClosedReason::DetachedRoot,
        };
        return ShadowExtraction::NotExtractable { reason };
    }

    ShadowExtraction::Extracted {
        children: hosts.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host(mode: ShadowDomMode, children: u32) -> ShadowHostObservation {
        ShadowHostObservation {
            frame_id: "F1".into(),
            host_token: format!("host-{}", children),
            mode,
            extractable_children: children,
        }
    }

    #[test]
    fn open_roots_are_extracted() {
        let hosts = vec![host(ShadowDomMode::Open, 3), host(ShadowDomMode::Open, 1)];
        match aggregate(&hosts) {
            ShadowExtraction::Extracted { children } => assert_eq!(children.len(), 2),
            other => panic!("expected Extracted, got {:?}", other),
        }
    }

    #[test]
    fn closed_root_is_reported_not_guessed() {
        let hosts = vec![host(ShadowDomMode::Open, 3), host(ShadowDomMode::Closed, 0)];
        match aggregate(&hosts) {
            ShadowExtraction::NotExtractable { reason } => {
                assert_eq!(reason, ClosedReason::ClosedRoot)
            }
            other => panic!("expected NotExtractable, got {:?}", other),
        }
    }

    #[test]
    fn detached_root_is_reported() {
        let hosts = vec![host(ShadowDomMode::Detached, 0)];
        match aggregate(&hosts) {
            ShadowExtraction::NotExtractable { reason } => {
                assert_eq!(reason, ClosedReason::DetachedRoot)
            }
            other => panic!("expected NotExtractable, got {:?}", other),
        }
    }
}
