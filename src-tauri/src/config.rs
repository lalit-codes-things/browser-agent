/// Runtime configuration surfaces.
///
/// Security-critical undefined parameters must not receive
/// frontend-authored defaults. Values that are undefined in the
/// catalog remain undefined here as Option and are surfaced before
/// any release-gating decision.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RuntimeConfig {
    pub model_manifest_path: String,
    pub installed_models_dir: String,
    pub audit_segment_dir: String,
    pub quarantine_dir: String,
    pub profiles_dir: String,
    pub egress_mode: EgressMode,
    pub max_steps: Option<u32>,             // undefined in catalog (C-149)
    pub max_llm_calls: Option<u32>,         // undefined in catalog (C-149)
    pub max_recovery_attempts: Option<u32>, // undefined in catalog (C-149)
    pub max_task_duration_ms: Option<u64>,  // undefined in catalog (C-149)
    pub max_confirmations: Option<u32>,     // undefined in catalog (C-149)
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum EgressMode {
    Enforced,
    Disabled,
}
