use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPin {
    pub artifact_id: String,
    pub model: ModelIdentity,
    pub source: ModelSource,
    pub sha256: String,
    pub bytes: u64,
    pub phase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelIdentity {
    pub family: String,
    pub format: String,
    pub quantization: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSource {
    pub repository: String,
    pub file: String,
}
