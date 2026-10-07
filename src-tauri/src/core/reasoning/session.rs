// Inference session.
//
// C-64: Phase-1 slice invokes model deterministically at every step using
//        fixed model, prompt/schema, quantization, runtime, decoding
//        configuration, and seed.
// C-154: orchestrator-owned progress supplied to model at every step.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceSession {
    pub task_id: String,
    pub epoch: u64,
    pub progress: crate::core::orchestrator::progress::TaskProgress,
    pub deterministic_config: DeterministicConfig,
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicConfig {
    pub model_family: String,
    pub quantization: String,
    pub decoding: DecodingMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum DecodingMode {
    Deterministic,
    Sampling, // separate experiment only (C-57)
}
