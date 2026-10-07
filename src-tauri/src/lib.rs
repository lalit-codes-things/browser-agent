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
// Where a phase is scheduled but not yet delivered, we use explicit
// gated stubs and NOT_IMPLEMENTED_YET markers rather than fake success
// paths (per IDE prompt rule 8).

pub mod error;
pub mod config;
pub mod ipc;
pub mod app;

#[cfg(feature = "runtime-core")]
pub mod core;
#[cfg(feature = "runtime-core")]
pub mod cdp;
#[cfg(feature = "runtime-core")]
pub mod browser;
#[cfg(feature = "runtime-core")]
pub mod net;
#[cfg(feature = "runtime-core")]
pub mod inference;
#[cfg(feature = "runtime-core")]
pub mod audit;
#[cfg(feature = "runtime-core")]
pub mod storage;
#[cfg(feature = "runtime-core")]
pub mod security;

pub use app::lifecycle::*;
pub use app::settings::*;
pub use app::diagnostics::*;
pub use config::RuntimeConfig;
pub use error::Error;
