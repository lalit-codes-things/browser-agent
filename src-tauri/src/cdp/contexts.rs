// Contexts.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub context_id: String,
    pub frame_id: String,
    pub origin: String,
}

pub struct ContextIndex;

impl ContextIndex {
    pub fn context_for_frame(_frame_id: &str) -> Result<Option<ExecutionContext>, crate::Error> {
        Err(crate::Error::NotImplemented("ContextIndex::context_for_frame is scheduled".into()))
    }
}
