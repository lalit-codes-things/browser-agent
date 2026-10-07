// Behavioral canary.
//
// C-44: every model-file change is gated by models/canary/behavioral-tests.json;
//        hash pinning alone is insufficient to prove behavioral integrity.
//
// The pass threshold is undefined in Revision 2 (TUNABLE). Do not invent it.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryBattery {
    pub tests: Vec<CanaryTest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryTest {
    pub id: String,
    pub description: String,
    pub pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryRunResult {
    pub run_id: String,
    pub passed_count: u32,
    pub total_count: u32,
    pub pass_threshold: Option<f64>,
    pub passed: Option<bool>,
}

impl CanaryRunResult {
    pub fn evaluate(&self) -> Option<bool> {
        let threshold = self.pass_threshold?;
        let ratio = self.passed_count as f64 / self.total_count.max(1) as f64;
        Some(ratio >= threshold)
    }
}
