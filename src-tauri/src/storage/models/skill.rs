// Stored skill model.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSkill {
    pub id: String,
    pub version: String,
    pub status: crate::core::skills::store::SkillStatus,
    pub commitment_hash: String,
    pub memory: Vec<String>,
}
