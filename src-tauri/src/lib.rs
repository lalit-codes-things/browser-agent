// Browser Agent — local-first browser automation runtime
//
// Architecture (C-01, C-138):
//   5 engines -> orchestrator-owned progress -> semantic state graph
//   -> deterministic baseline -> tiered policy -> typed execution
//   -> independent verification -> immutable skills
//
// Security posture:
//   - Rust owns task authority, policy, authorization tier, verification,
//     epochs, taint/provenance, secrets, egress control, model integrity,
//     audit chain, hard stops, recovery decisions.
//   - Frontend is a projection of typed events. It does not invent
//     security truth (C-17, C-18).
//
// This file is the crate root; subsystem modules live under src/.
// The `runtime-core` feature gates subsystems that depend on the browser
// runtime being attached; app-shell code compiles without it.
// Where a phase is scheduled but not yet delivered, we use explicit
// gated stubs and NOT_IMPLEMENTED_YET markers rather than fake success
// paths (per IDE prompt rule 8).

pub mod error;
pub mod config;
pub mod ipc;
pub mod app;

#[cfg(any(test, feature = "runtime-core"))]
pub mod core;
#[cfg(any(test, feature = "runtime-core"))]
pub mod cdp;
#[cfg(any(test, feature = "runtime-core"))]
pub mod browser;
#[cfg(any(test, feature = "runtime-core"))]
pub mod net;
#[cfg(any(test, feature = "runtime-core"))]
pub mod inference;
#[cfg(any(test, feature = "runtime-core"))]
pub mod audit;
#[cfg(any(test, feature = "runtime-core"))]
pub mod storage;
#[cfg(any(test, feature = "runtime-core"))]
pub mod security;

pub use app::lifecycle::*;
pub use app::settings::*;
pub use app::diagnostics::*;
pub use config::RuntimeConfig;
pub use error::Error;

/// Runtime initialization entry point.
///
/// Real Tauri bootstrap (window creation, command registration, event
/// emission) is wired in the Phase 1 vertical slice. This function exists
/// so the binary has an explicit, testable init boundary rather than a
/// fake success path.
pub fn init() {
    tracing::debug!("browser-agent runtime core initialized");
}

#[cfg(test)]
mod tests {
    #[test]
    fn init_is_idempotent_and_does_not_panic() {
        super::init();
        super::init();
    }
}
