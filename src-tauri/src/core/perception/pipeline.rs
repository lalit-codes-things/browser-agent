// Perception pipeline.
//
// C-58/C-80: builds the bounded semantic state graph from frame
//        observations across the A11Y/DOM rungs (C-84).
// C-59: stabilization gates action; livelock detection upstream.
// C-83: compression bounded by supplied caps.
// C-141: pipeline output carries the epoch identity actions bind to.
//
// CDP-backed capture is scheduled with the browser runtime; the
// aggregation/compression/epoch logic below is fully live.

use crate::core::perception::compress::{compress, CompressOutcome};
use crate::core::perception::epoch::PerceptionEpoch;
use crate::core::perception::frames::{
    aggregate_frames, epoch_for, AggregationError, FrameObservation,
};
use crate::core::perception::graph::{GraphBounds, SemanticStateGraph};

#[derive(Debug, Clone)]
pub struct PipelineOutput {
    pub graph: SemanticStateGraph,
    pub epoch: PerceptionEpoch,
    /// Number of actionable nodes after compression.
    pub actionable_count: usize,
    /// Caps were exceeded: summary is explicitly bounded, callers must
    /// not treat it as complete perception.
    pub bounded_exceeded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineError {
    Aggregation(AggregationError),
    NotImplemented,
}

impl From<AggregationError> for PipelineError {
    fn from(e: AggregationError) -> Self {
        PipelineError::Aggregation(e)
    }
}

/// Epoch identity for the pipeline run: frame/loader identity from CDP,
/// value supplied by the orchestrator's epoch counter.
pub fn build(
    frames: &[FrameObservation],
    epoch_value: u64,
    session_id: String,
    caps: &GraphBounds,
) -> Result<PipelineOutput, PipelineError> {
    let mut graph = aggregate_frames(frames)?;
    graph.session_id = session_id;
    graph.epoch = epoch_value;

    let main = frames
        .iter()
        .find(|f| f.is_main_frame)
        .ok_or(PipelineError::Aggregation(
            AggregationError::MissingMainFrame,
        ))?;
    let epoch = epoch_for(main, epoch_value);

    match compress(&graph.nodes, caps) {
        CompressOutcome::Compressed(summary) => Ok(PipelineOutput {
            graph,
            epoch,
            actionable_count: summary.actionable.len(),
            bounded_exceeded: false,
        }),
        CompressOutcome::BoundedExceeded { .. } => {
            // Graph is still returned; the bound exceedance is surfaced
            // explicitly and the caller decides (re-perceive, descend
            // rungs, or stop). We do not silently drop nodes.
            Ok(PipelineOutput {
                graph,
                epoch,
                actionable_count: 0,
                bounded_exceeded: true,
            })
        }
    }
}

/// Live CDP capture boundary: wired with the browser runtime (Phase 2/3).
///
/// This is the contract between the browser runtime controller and the
/// perception pipeline. When the runtime is attached and the CDP transport is
/// ready, this path becomes the real capture entry point.
pub fn capture(runtime: &crate::browser::controller::BrowserRuntimeSnapshot) -> Result<Vec<FrameObservation>, PipelineError> {
    if !runtime.available() {
        return Err(PipelineError::NotImplemented);
    }
    Err(PipelineError::NotImplemented)
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
            bounds: Some(GeometryBounds {
                x: 0,
                y: 0,
                width: 4,
                height: 4,
            }),
        }
    }

    fn frames() -> Vec<FrameObservation> {
        vec![
            FrameObservation {
                frame_id: "M".into(),
                loader_id: "L-M".into(),
                origin: "https://example.test".into(),
                is_main_frame: true,
                is_oopif: false,
                nodes: vec![node("button"), node("link")],
            },
            FrameObservation {
                frame_id: "IF1".into(),
                loader_id: "L-IF1".into(),
                origin: "https://frame.test".into(),
                is_main_frame: false,
                is_oopif: true,
                nodes: vec![node("textbox")],
            },
        ]
    }

    fn caps() -> GraphBounds {
        GraphBounds::new()
    }

    #[test]
    fn builds_graph_and_epoch() {
        let out = build(&frames(), 42, "S1".into(), &caps()).unwrap();
        assert_eq!(out.epoch.value, 42);
        assert_eq!(out.epoch.frame_id, "M");
        assert_eq!(out.epoch.loader_id, "L-M");
        assert_eq!(out.graph.epoch, 42);
        assert_eq!(out.graph.session_id, "S1");
        assert_eq!(out.actionable_count, 3);
        assert!(!out.bounded_exceeded);
    }

    #[test]
    fn bound_exceedance_is_surfaced() {
        let tight = GraphBounds {
            max_nodes: Some(1),
            max_context_bytes: None,
            max_attributes_per_node: None,
        };
        let out = build(&frames(), 1, "S1".into(), &tight).unwrap();
        assert!(out.bounded_exceeded);
        assert_eq!(out.actionable_count, 0);
        // Graph identity is still present.
        assert_eq!(out.epoch.value, 1);
    }

    #[test]
    fn propagates_aggregation_errors() {
        let bad = vec![frames().remove(1)];
        assert!(matches!(
            build(&bad, 1, "S1".into(), &caps()),
            Err(PipelineError::Aggregation(_))
        ));
    }
}
