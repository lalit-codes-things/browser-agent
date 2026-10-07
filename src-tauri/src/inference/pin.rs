// Model pin metadata.
//
// C-37: model artifact manifest and SHA256 identity stored in ordinary Git;
//        multi-GB model binaries in Git LFS or signed release artifact.
// C-41: SHA-256 verified at every model load.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPin {
    pub artifact_id: String,
    pub family: String,
    pub format: String,
    pub quantization: String,
    pub sha256: String,
    pub bytes: u64,
    pub source_revision: Option<String>,
    pub phase: String,
}
