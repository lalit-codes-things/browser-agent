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
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskLevel {
    Low,
    Moderate,
    High,
    Unknown,
}

pub struct TaskNormalizer;

impl TaskNormalizer {
    pub fn normalize(task_text: &str) -> Result<NormalizedTask, crate::Error> {
        let text = task_text.trim();
        if text.is_empty() || text.len() > 4_000 {
            return Err(crate::Error::InvalidParameter(
                "task text must be 1..4000 bytes".into(),
            ));
        }
        let normalized_goal = text.split_whitespace().collect::<Vec<_>>().join(" ");
        let lower = normalized_goal.to_ascii_lowercase();
        let high_stakes = [
            "pay",
            "purchase",
            "transfer",
            "send money",
            "delete account",
        ]
        .iter()
        .any(|needle| lower.contains(needle));
        Ok(NormalizedTask {
            id: format!("task-{}", crate::next_task_id()),
            original_text: text.to_owned(),
            normalized_goal,
            sites: Vec::new(),
            items: vec![TaskItem {
                description: text.to_owned(),
                item_type: "USER_GOAL".into(),
            }],
            constraints: Vec::new(),
            capabilities_required: Vec::new(),
            projected_risk: ProjectedRisk {
                classification: if high_stakes {
                    "HIGH_STAKES"
                } else {
                    "GENERAL"
                }
                .into(),
                level: if high_stakes {
                    RiskLevel::High
                } else {
                    RiskLevel::Unknown
                },
            },
        })
    }
}
