// Locale-aware amount handling.
//
// C-08, C-15, C-33: amount thresholds and locale-aware bounded amount deltas
//        are security-relevant. Values are backend-provided.
//
// The frontend must render amounts verbatim from the backend (C-20-style
// copy discipline). We do not invent currency formatting here.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountDisplay {
    pub raw: String,
    pub currency: Option<String>,
}

impl AmountDisplay {
    pub fn new(raw: String, currency: Option<String>) -> Self {
        Self { raw, currency }
    }
}
