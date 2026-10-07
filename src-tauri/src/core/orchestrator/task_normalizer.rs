// Task Normalizer.
//
// Translates a natural-language user task into a typed task authority
// object: normalized goal, sites, items, constraints, capabilities
// required, projected risk/classification.
//
// The model does not invent task authority here. This module produces a
// structured, auditable task definition that later stages consume.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedTask {
    pub id: String,
    pub original_text: String,
    pub normalized_goal: String,
    pub sites: Vec<String>,
    pub items: Vec<TaskItem>,
    pub constraints: Vec<String>,
    pub capabilities_required: Vec<String>,
    pub projected_risk: ProjectedRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskItem {
    pub description: String,
    pub item_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectedRisk {
    pub classification: String,
    pub level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RiskLevel {
    Low,
    Moderate,
    High,
    Unknown,
}

pub struct TaskNormalizer;

impl TaskNormalizer {
    pub fn normalize(_task_text: &str) -> Result<NormalizedTask, crate::Error> {
        // Phase 1 placeholder. Real normalization is backend-driven and
        // auditable. We do not fake a complete normalizer.
        Err(crate::Error::NotImplemented(
            "TaskNormalizer::normalize is scheduled, not implemented yet".into(),
        ))
    }
}
