/// Runtime configuration surfaces.
///
/// Security-critical undefined parameters must not receive
/// frontend-authored defaults. Values that are undefined in the
/// catalog remain undefined here as Option and are surfaced before
/// any release-gating decision.
///
/// C-04: Chromium is launched with --remote-debugging-pipe. The pinned
/// executable path is undefined in the catalog until a real browser is
/// pinned for the local slice; when absent the runtime fails closed with
/// BROWSER_RUNTIME_UNAVAILABLE.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RuntimeConfig {
    pub model_manifest_path: String,
    pub installed_models_dir: String,
    pub audit_segment_dir: String,
    pub quarantine_dir: String,
    pub profiles_dir: String,
    pub egress_mode: EgressMode,
    /// Pinned Chromium executable path (C-04). Undefined in catalog until
    /// a real browser is pinned; absent means the runtime reports
    /// BROWSER_RUNTIME_UNAVAILABLE.
    pub chromium_executable: Option<String>,
    pub max_steps: Option<u32>,             // undefined in catalog (C-149)
    pub max_llm_calls: Option<u32>,         // undefined in catalog (C-149)
    pub max_recovery_attempts: Option<u32>, // undefined in catalog (C-149)
    pub max_task_duration_ms: Option<u64>,  // undefined in catalog (C-149)
    pub max_confirmations: Option<u32>,     // undefined in catalog (C-149)
}

impl RuntimeConfig {
    /// Returns the pinned Chromium executable path, if one is pinned.
    pub fn chromium_executable_path(&self) -> Option<&std::path::Path> {
        self.chromium_executable
            .as_deref()
            .map(std::path::Path::new)
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum EgressMode {
    Enforced,
    Disabled,
}
