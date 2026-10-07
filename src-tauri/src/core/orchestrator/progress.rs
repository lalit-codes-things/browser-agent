// Task Progress.
//
// Orchestrator-owned, updated deterministically after each verified action
// and supplied to the model at every step (C-154). The frontend projects
// this object; it does not invent progress.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    pub task_id: String,
    pub current_state: TaskState,
    pub step_index: u32,
    pub total_steps: u32,
    pub epoch: u64,
    pub mode: ProgressMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TaskState {
    Pending,
    Running,
    Verified,
    LikelySuccess,
    Unknown,
    Failed,
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProgressMode {
    DeterministicSkill,
    ModelReasoning,
    Intervention,
    Recovery,
    Parked,
}
