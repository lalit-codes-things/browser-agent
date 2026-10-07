// Application lifecycle.
//
// Phase 0 engineering concern: we keep ADRs, risk register, isolation
// matrix, and catalog integration under docs/ and tools/. This module
// is the runtime-side anchor for those artifacts rather than a place
// to hide implementation gaps.

pub const APP_ID: &str = "com.browseragent.app";

/// Minimal application metadata for runtime surfaces.
pub fn app_identity() -> AppIdentity {
    AppIdentity {
        id: APP_ID.into(),
        name: "Browser Agent".into(),
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppIdentity {
    pub id: String,
    pub name: String,
}
