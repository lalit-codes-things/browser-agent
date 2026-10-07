// Grammar-constrained decoding metadata.
//
// C-47: structured inference p95 per proposal is <= 2.0 seconds.
// C-151: metrics separate schema-valid rate from semantically-correct-action
//         rate; grammar-constrained decoding overhead is measured separately.
//
// We surface the grammar constraint concept here rather than pretending
// a complete grammar engine exists in Phase 1.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarConstraint {
    pub name: String,
    pub schema_ref: String,
}
