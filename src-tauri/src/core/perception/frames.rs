// Frame and OOPIF aggregation.
//
// C-80: per-frame perception with OOPIF aggregation. Cross-origin frames
//        are separate processes; their graphs merge into one semantic
//        state graph with frame identity preserved.
//
// Frame/loader identity data comes from the CDP runtime (cdp::frames,
// cdp::loaders). This module owns the aggregation semantics.

use serde::{Deserialize, Serialize};

use crate::core::perception::epoch::PerceptionEpoch;
use crate::core::perception::graph::{FrameNode, GraphNode, SemanticStateGraph};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameObservation {
    pub frame_id: String,
    pub loader_id: String,
    pub origin: String,
    pub is_main_frame: bool,
    /// OOPIF (out-of-process iframe) classification from CDP.
    pub is_oopif: bool,
    pub nodes: Vec<GraphNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregationError {
    /// Duplicate frame in the observation set: extraction bug, not user
    /// error. Caller must re-perceive rather than merge blindly.
    DuplicateFrame,
    /// No main frame present: cannot anchor a graph.
    MissingMainFrame,
}

pub fn aggregate_frames(frames: &[FrameObservation]) -> Result<SemanticStateGraph, AggregationError> {
    if frames.is_empty() {
        return Err(AggregationError::MissingMainFrame);
    }

    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    for f in frames {
        if !seen.insert((f.frame_id.clone(), f.loader_id.clone())) {
            return Err(AggregationError::DuplicateFrame);
        }
    }

    let main = frames.iter().find(|f| f.is_main_frame).ok_or(AggregationError::MissingMainFrame)?;

    let mut nodes = Vec::new();
    for f in frames {
        nodes.extend(f.nodes.iter().cloned());
    }

    Ok(SemanticStateGraph {
        nodes,
        frames: frames
            .iter()
            .map(|f| FrameNode {
                frame_id: f.frame_id.clone(),
                loader_id: f.loader_id.clone(),
                origin: f.origin.clone(),
                is_main_frame: f.is_main_frame,
                child_frame_count: frames.iter().filter(|o| !o.is_main_frame).count() as u32,
            })
            .collect(),
        session_id: String::new(),
        loader_id: main.loader_id.clone(),
        epoch: 0,
    })
}

/// Build the perception epoch identity for a frame observation set.
/// Epoch value is assigned by the pipeline; frame identity comes from CDP.
pub fn epoch_for(main: &FrameObservation, epoch_value: u64) -> PerceptionEpoch {
    PerceptionEpoch::new(epoch_value, main.frame_id.clone(), main.loader_id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::perception::graph::{GeometryBounds, GraphNode};

    fn node(role: &str) -> GraphNode {
        GraphNode {
            role: role.into(),
            label: Some(role.into()),
            rendered_text: None,
            accessible_name: None,
            actionable: true,
            bounds: Some(GeometryBounds { x: 0, y: 0, width: 4, height: 4 }),
        }
    }

    fn frame(id: &str, main: bool, oopif: bool) -> FrameObservation {
        FrameObservation {
            frame_id: id.into(),
            loader_id: format!("L-{}", id),
            origin: "https://example.test".into(),
            is_main_frame: main,
            is_oopif: oopif,
            nodes: vec![node("button")],
        }
    }

    #[test]
    fn aggregates_main_and_oopif() {
        let frames = vec![frame("M", true, false), frame("IF1", false, true)];
        let graph = aggregate_frames(&frames).unwrap();
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.frames.len(), 2);
        assert_eq!(graph.loader_id, "L-M");
    }

    #[test]
    fn duplicate_frame_is_rejected() {
        let frames = vec![frame("M", true, false), frame("M", true, false)];
        assert!(matches!(aggregate_frames(&frames), Err(AggregationError::DuplicateFrame)));
    }

    #[test]
    fn missing_main_frame_is_rejected() {
        let frames = vec![frame("IF1", false, true)];
        assert!(matches!(aggregate_frames(&frames), Err(AggregationError::MissingMainFrame)));
    }

    #[test]
    fn empty_observation_is_rejected() {
        assert!(matches!(aggregate_frames(&[]), Err(AggregationError::MissingMainFrame)));
    }
}
