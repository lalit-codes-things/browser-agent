// Accessibility tree extraction.
//
// C-80: per-frame perception; the A11Y rung is the preferred perception
//        path (C-84 fallback ladder).
// C-86: accessible-name disagreements flow to deception.rs, not here.
//
// In Phase 1 this module defines the extraction contract and the
// adapter boundary for CDP. The CDP-backed extraction itself is wired
// when the browser runtime attaches; the module fails with
// NotImplemented rather than pretending extraction happened.

use serde::{Deserialize, Serialize};

use crate::core::perception::graph::GraphNode;

/// Raw accessibility node as reported by the browser's AX tree.
/// Field names mirror the CDP AX node shape we consume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxAbsractNode {
    pub ax_node_id: String,
    pub role: Option<String>,
    pub name: Option<String>,
    pub value: Option<String>,
    pub bounds: Option<crate::core::perception::graph::GeometryBounds>,
    pub ignored: bool,
    pub frame_id: String,
}

/// Convert an AX node into a graph node. `ignored` AX nodes are dropped:
/// they are presentation-only and must not become action candidates.
pub fn to_graph_node(ax: &AxAbsractNode) -> Option<GraphNode> {
    if ax.ignored {
        return None;
    }
    Some(GraphNode {
        role: ax.role.clone().unwrap_or_default(),
        label: ax.name.clone(),
        rendered_text: ax.value.clone(),
        accessible_name: ax.name.clone(),
        actionable: matches!(
            ax.role.as_deref(),
            Some("button") | Some("link") | Some("textbox") | Some("searchbox") | Some("combobox") | Some("listbox")
        ),
        bounds: ax.bounds.clone(),
    })
}

/// Rung adapter stub: real CDP AX extraction is Phase 3; the rung
/// identity and failure mode are fixed now.
pub fn extract_via_cdp(_frame_id: &str) -> Result<Vec<GraphNode>, crate::Error> {
    Err(crate::Error::NotImplemented(
        "ax_extract: CDP-backed AX extraction scheduled (Phase 3)".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::perception::graph::GeometryBounds;

    #[test]
    fn ignored_nodes_are_dropped() {
        let ax = AxAbsractNode {
            ax_node_id: "1".into(),
            role: Some("button".into()),
            name: Some("Pay".into()),
            value: None,
            bounds: Some(GeometryBounds { x: 0, y: 0, width: 4, height: 4 }),
            ignored: true,
            frame_id: "F1".into(),
        };
        assert!(to_graph_node(&ax).is_none());
    }

    #[test]
    fn actionable_roles_map() {
        let ax = AxAbsractNode {
            ax_node_id: "2".into(),
            role: Some("textbox".into()),
            name: Some("Amount".into()),
            value: None,
            bounds: None,
            ignored: false,
            frame_id: "F1".into(),
        };
        let n = to_graph_node(&ax).unwrap();
        assert!(n.actionable);
        assert_eq!(n.accessible_name.as_deref(), Some("Amount"));
    }

    #[test]
    fn non_actionable_role_maps() {
        let ax = AxAbsractNode {
            ax_node_id: "3".into(),
            role: Some("banner".into()),
            name: None,
            value: None,
            bounds: None,
            ignored: false,
            frame_id: "F1".into(),
        };
        let n = to_graph_node(&ax).unwrap();
        assert!(!n.actionable);
    }

    #[test]
    fn missing_role_defaults_empty_string() {
        let ax = AxAbsractNode {
            ax_node_id: "4".into(),
            role: None,
            name: None,
            value: None,
            bounds: None,
            ignored: false,
            frame_id: "F1".into(),
        };
        let n = to_graph_node(&ax).unwrap();
        assert_eq!(n.role, "");
    }
}
