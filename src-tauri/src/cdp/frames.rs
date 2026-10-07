// Frames.
//
// C-58, C-80, C-87: frame and OOPIF tracking with recursive hit-testing.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameDescription {
    pub frame_id: String,
    pub loader_id: String,
    pub parent_frame_id: Option<String>,
    pub url: String,
    pub is_main_frame: bool,
    pub security_origin: String,
}

pub struct FrameIndex;

impl FrameIndex {
    pub fn frame_for_point(_frame_id: &str, _x: f64, _y: f64) -> Result<Option<FrameDescription>, crate::Error> {
        Err(crate::Error::NotImplemented("FrameIndex::frame_for_point is scheduled".into()))
    }
}
