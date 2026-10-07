// Skill versions.
//
// C-100: skill records and skill versions are immutable.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillVersion {
    pub skill_id: String,
    pub version_id: String,
    pub commitment_hash: String,
    pub created_at: Option<String>,
}
