// Diagnostics.
//
// C-98: latency has dual gates: per-proposal inference p95 and end-to-end
//        step p95.
// C-151: metrics separate schema-valid rate from semantically-correct-action
//        rate; grammar-constrained decoding overhead measured separately.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    pub inference_p95_ms: Option<u64>,
    pub step_p95_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub schema_valid_count: u64,
    pub schema_valid_total: u64,
    pub semantically_correct_count: u64,
    pub semantically_correct_total: u64,
    pub grammar_overhead_ms: u64,
}

pub struct Diagnostics;

impl Diagnostics {
    pub fn latency(&self) -> LatencyMetrics {
        LatencyMetrics {
            inference_p95_ms: None,
            step_p95_ms: None,
        }
    }

    pub fn quality(&self) -> QualityMetrics {
        QualityMetrics {
            schema_valid_count: 0,
            schema_valid_total: 0,
            semantically_correct_count: 0,
            semantically_correct_total: 0,
            grammar_overhead_ms: 0,
        }
    }
}
