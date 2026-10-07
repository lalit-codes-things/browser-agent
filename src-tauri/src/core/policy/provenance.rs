// Provenance / data flow.
//
// C-31: sensitive-to-public data flow maps explicitly to HIGH_STAKES;
//        credential-to-public is blocked.
// C-36: trusted runtime data composes confirmation; LLM/evidence logic
//        is never the authority for HIGH_STAKES.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum DataFlowClass {
    TrustedRuntime,
    PageDerived,
    SensitiveToPublic,
    CredentialToPublic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowSummary {
    pub source: String,
    pub sink: String,
    pub flow_class: DataFlowClass,
    pub verdict: Option<String>,
}
