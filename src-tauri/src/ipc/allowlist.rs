// IPC allowlist metadata.
//
// This documents the intent behind the allowlisted surface. The actual
// enforcement happens through Tauri capabilities and typed command/event
// boundaries.
//
// Rule: no arbitrary object blobs for security-sensitive commands.
// (C-16)

pub const IPC_CATEGORY_TASKS: &str = "tasks";
pub const IPC_CATEGORY_MODEL: &str = "model";
pub const IPC_CATEGORY_POLICY: &str = "policy";
pub const IPC_CATEGORY_VERIFICATION: &str = "verification";
pub const IPC_CATEGORY_SECURITY: &str = "security";
