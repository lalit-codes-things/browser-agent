// DOM extraction.
//
// C-80: per-frame perception. The DOM rung is the second rung of the
//        C-84 fallback ladder, used when the AX tree is insufficient.
//
// The extraction contract mirrors ax_extract. The CDP-backed traversal
// itself is wired when the browser runtime attaches.

use serde::{Deserialize, Serialize};

use crate::core::perception::graph::GraphNode;

/// Raw DOM node as reported by the browser runtime traversal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomRawNode {
    pub dom_node_id: String,
    pub tag: Option<String>,
    pub aria_role: Option<String>,
    pub text: Option<String>,
    pub bounds: Option<crate::core::perception::graph::GeometryBounds>,
    pub frame_id: String,
}

/// Map a DOM node to a graph node. Nodes without an implicit or explicit
/// ARIA role map to an empty role and are not actionable candidates.
pub fn to_graph_node(dom: &DomRawNode) -> Option<GraphNode> {
    let role = dom.aria_role.clone().unwrap_or_else(|| "".into());
    Some(GraphNode {
        role,
        label: dom.text.clone(),
        rendered_text: dom.text.clone(),
        accessible_name: None, // DOM rung has no AX name; absence is explicit.
        actionable: matches!(
            dom.aria_role.as_deref(),
            Some("button") | Some("link") | Some("textbox") | Some("searchbox") | Some("combobox") | Some("listbox")
        ),
        bounds: dom.bounds.clone(),
    })
}

/// Rung adapter stub: real CDP DOM traversal is Phase 3.
pub fn extract_via_cdp(_frame_id: &str) -> Result<Vec<GraphNode>, crate::Error> {
    Err(crate::Error::NotImplemented(
        "dom_extract: CDP-backed DOM traversal scheduled (Phase 3)".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::perception::graph::GeometryBounds;

    #[test]
    fn maps_button_node() {
        let dom = DomRawNode {
            dom_node_id: "9".into(),
            tag: Some("button".into()),
            aria_role: Some("button".into()),
            text: Some("Submit".into()),
            bounds: Some(GeometryBounds { x: 0, y: 0, width: 4, height: 4 }),
            frame_id: "F1".into(),
        };
        let n = to_graph_node(&dom).unwrap();
        assert!(n.actionable);
        assert_eq!(n.rendered_text.as_deref(), Some("Submit"));
        assert!(n.accessible_name.is_none());
    }

    #[test]
    fn missing_role_is_not_actionable() {
        let dom = DomRawNode {
            dom_node_id: "10".into(),
            tag: Some("div".into()),
            aria_role: None,
            text: Some("hello".into()),
            bounds: None,
            frame_id: "F1".into(),
        };
        let n = to_graph_node(&dom).unwrap();
        assert!(!n.actionable);
        assert_eq!(n.role, "");
    }

    #[test]
    fn dom_rung_never_fabricates_ax_names() {
        let dom = DomRawNode {
            dom_node_id: "11".into(),
            tag: Some("a".into()),
            aria_role: Some("link".into()),
            text: Some("Docs".into()),
            bounds: None,
            frame_id: "F1".into(),
        };
        let n = to_graph_node(&dom).unwrap();
        assert!(n.accessible_name.is_none(), "DOM rung must not invent AX names");
    }
}
