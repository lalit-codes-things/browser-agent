// Bounded semantic state graph.
//
// C-58: perception must track target/session/frame/loader identity and
//        maintain a bounded semantic state graph.
// C-83: graph caps exist; exact numeric cap values are undefined in
//        Revision 2 and are not invented here.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticStateGraph {
    pub nodes: Vec<GraphNode>,
    pub frames: Vec<FrameNode>,
    pub session_id: String,
    pub loader_id: String,
    pub epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphNode {
    pub role: String,
    pub label: Option<String>,
    pub rendered_text: Option<String>,
    pub accessible_name: Option<String>,
    pub actionable: bool,
    pub bounds: Option<GeometryBounds>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameNode {
    pub frame_id: String,
    pub loader_id: String,
    pub origin: String,
    pub is_main_frame: bool,
    pub child_frame_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeometryBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub struct GraphBounds {
    pub max_nodes: Option<usize>,
    pub max_context_bytes: Option<usize>,
    pub max_attributes_per_node: Option<usize>,
}

impl GraphBounds {
    pub fn new() -> Self {
        Self {
            max_nodes: None,
            max_context_bytes: None,
            max_attributes_per_node: None,
        }
    }
}

impl Default for GraphBounds {
    fn default() -> Self {
        Self::new()
    }
}
