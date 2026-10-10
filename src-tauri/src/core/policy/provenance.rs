use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum DataFlowClass {
    #[default]
    TrustedRuntime,
    PageDerived,
    SensitiveToPublic,
    CredentialToPublic,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataFlowSummary {
    pub source: String,
    pub sink: String,
    pub flow_class: DataFlowClass,
    pub verdict: Option<String>,
}
