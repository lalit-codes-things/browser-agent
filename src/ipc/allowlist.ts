// Frontend allowlist documentation.
//
// The actual allowlist is enforced server-side via Tauri capabilities and
// the typed command registry in src-tauri/src/ipc/allowlist.rs.
//
// Rule: no arbitrary object blobs for security-sensitive commands.
// (C-16)
