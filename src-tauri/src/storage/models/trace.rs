// Stored trace model.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTrace {
    pub id: String,
    pub task_id: String,
    pub events: Vec<String>,
    pub created_at: Option<String>,
}
