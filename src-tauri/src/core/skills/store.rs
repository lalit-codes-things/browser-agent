// Skill store.
//
// C-100: skill records and skill versions are immutable; macro execution is
//        represented through canonical semantic SkillCommitments.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRecord {
    pub id: String,
    pub version: String,
    pub origin_scope: String,
    pub status: SkillStatus,
    pub commitment_hash: String,
    pub last_shadow_eval: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SkillStatus {
    Active,
    Shadow,
    SuspendedDrift,
    Revoked,
}

pub struct SkillStore;

impl SkillStore {
    pub fn record(_id: &str, _version: &str, _origin_scope: &str, _commitment_hash: &str) -> SkillRecord {
        SkillRecord {
            id: _id.into(),
            version: _version.into(),
            origin_scope: _origin_scope.into(),
            status: SkillStatus::Active,
            commitment_hash: _commitment_hash.into(),
            last_shadow_eval: None,
        }
    }
}
