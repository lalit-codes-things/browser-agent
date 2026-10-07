// IPC boundary.
//
// All IPC commands/events must be explicit and typed. The frontend
// never calls privileged capabilities directly; commands route through
// the allowlisted Tauri IPC layer.
//
// This is a placeholder stub for the typed command registry. In Phase 1
// we flesh out the command/ event types and add allowlist enforcement.

pub mod commands;
pub mod events;
pub mod allowlist;
pub mod errors;
