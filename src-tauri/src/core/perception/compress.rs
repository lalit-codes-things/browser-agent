// Semantic compression.
//
// C-83: semantic compression bounded by graph/context/attribute caps.
//
// Cap values are TUNABLE/undefined in Revision 2 and are supplied by the
// caller (GraphBounds). This module never invents defaults.

use crate::core::perception::graph::{GraphBounds, GraphNode};

#[derive(Debug, Clone, PartialEq)]
pub enum CompressOutcome {
    /// Compression completed within caps.
    Compressed(SemanticSummary),
    /// Caps were exhausted before a faithful summary could be produced.
    /// The caller receives an explicit bounded result, not a truncated
    /// silent summary.
    BoundedExceeded { nodes_dropped: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticSummary {
    /// Actionable nodes, most relevant first.
    pub actionable: Vec<GraphNode>,
    /// Total node count before compression.
    pub total_nodes: usize,
    /// Nodes retained after compression.
    pub retained: usize,
}

pub fn compress(nodes: &[GraphNode], caps: &GraphBounds) -> CompressOutcome {
    let total = nodes.len();

    // Undefined caps are honored as "no limit from config" — the
    // absence is explicit and surfaced, never replaced with a guess.
    let kept: Vec<GraphNode> = match caps.max_nodes {
        Some(max) => {
            let actionable: Vec<&GraphNode> = nodes.iter().filter(|n| n.actionable).collect();
            if actionable.len() > max {
                return CompressOutcome::BoundedExceeded {
                    nodes_dropped: actionable.len() - max,
                };
            }
            actionable.into_iter().cloned().collect()
        }
        None => nodes.iter().filter(|n| n.actionable).cloned().collect(),
    };

    CompressOutcome::Compressed(SemanticSummary {
        retained: kept.len(),
        total_nodes: total,
        actionable: kept,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::perception::graph::GeometryBounds;

    fn node(role: &str, actionable: bool) -> GraphNode {
        GraphNode {
            role: role.into(),
            label: Some(role.into()),
            rendered_text: None,
            accessible_name: None,
            actionable,
            bounds: Some(GeometryBounds {
                x: 0,
                y: 0,
                width: 10,
                height: 10,
            }),
        }
    }

    #[test]
    fn keeps_only_actionable() {
        let nodes = vec![
            node("button", true),
            node("banner", false),
            node("link", true),
        ];
        let caps = GraphBounds::new();
        match compress(&nodes, &caps) {
            CompressOutcome::Compressed(s) => {
                assert_eq!(s.actionable.len(), 2);
                assert_eq!(s.total_nodes, 3);
                assert_eq!(s.retained, 2);
            }
            other => panic!("expected Compressed, got {:?}", other),
        }
    }

    #[test]
    fn cap_exceeded_is_explicit_not_silent() {
        let nodes: Vec<GraphNode> = (0..10)
            .map(|i| node(&format!("button{}", i), true))
            .collect();
        let caps = GraphBounds {
            max_nodes: Some(4),
            max_context_bytes: None,
            max_attributes_per_node: None,
        };
        match compress(&nodes, &caps) {
            CompressOutcome::BoundedExceeded { nodes_dropped } => assert_eq!(nodes_dropped, 6),
            other => panic!("expected BoundedExceeded, got {:?}", other),
        }
    }

    #[test]
    fn undefined_cap_means_unbounded_not_zero() {
        let nodes = vec![node("button", true); 50];
        let caps = GraphBounds::new();
        match compress(&nodes, &caps) {
            CompressOutcome::Compressed(s) => assert_eq!(s.retained, 50),
            other => panic!("expected Compressed, got {:?}", other),
        }
    }
}
