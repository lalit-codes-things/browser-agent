// Stored task model.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTask {
    pub id: String,
    pub status: crate::core::orchestrator::progress::TaskState,
    pub progress: crate::core::orchestrator::progress::TaskProgress,
    pub preserved_state_summary: Option<String>,
}
