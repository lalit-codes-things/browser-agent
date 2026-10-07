// Skill commitments.
//
// C-100: macro execution is represented through canonical semantic
//        SkillCommitments.
// C-101: skill commitments are origin-bound by default and require unique,
//        safe resolution of required roles/relationships; fuzzy DOM fingerprint
//        similarity is prohibited.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCommitment {
    pub commitment_hash: String,
    pub origin: String,
    pub required_roles: Vec<String>,
    pub required_fields: Vec<String>,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
    pub invariants: Vec<String>,
    pub policy_requirements: Vec<String>,
}
