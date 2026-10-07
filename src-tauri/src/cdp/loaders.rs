// Loaders.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderDescription {
    pub loader_id: String,
    pub url: String,
    pub initiator: Option<String>,
    pub frame_id: String,
}

pub struct LoaderIndex;

impl LoaderIndex {
    pub fn loader_for_frame(_frame_id: &str) -> Result<Option<LoaderDescription>, crate::Error> {
        Err(crate::Error::NotImplemented("LoaderIndex::loader_for_frame is scheduled".into()))
    }
}
