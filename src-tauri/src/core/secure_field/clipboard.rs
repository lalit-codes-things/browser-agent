// Clipboard handling.
//
// Two distinct clipboard paths exist; they must never be conflated:
//
//   1. Agent-controlled secret handling.
//      The agent MUST NOT copy passwords, PAN, CVV, OTP, TOTP, UPI PIN,
//      recovery codes, or other HIGH_STAKES secrets to the normal macOS
//      pasteboard. There is no agent API such as
//        copy_secret_to_clipboard(...)
//      available to the autonomous execution path.
//
//   2. User-deliberate secret copy.
//      A deliberate user-facing vault operation may support copying a
//      permitted secret, but only via an explicit native capability, with
//      concealed/transient pasteboard metadata, shortest practical lifetime,
//      deterministic cleanup, and explicit platform-limitation notes.
//
// Universal Clipboard:
//   Concealed/Transient pasteboard metadata is best-effort clipboard
//   hygiene. It is NOT the security boundary for preventing system-level
//   pasteboard synchronization. We do NOT claim Universal Clipboard can be
//   disabled for one operation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipboardAction {
    /// Agent attempts to copy a secret. This is prohibited and must fail.
    AgentCopySecret,
    /// User-initiated copy of a permitted secret via the native capability.
    UserCopySecret,
    /// User-initiated copy of a non-secret (e.g. public label, safe text).
    UserCopyPublic,
    /// Deterministic cleanup requested.
    ClearClipboard,
}

/// Result of an agent-secret-copy attempt.
///
/// The invariant: the autonomous agent cannot place secrets onto the macOS
/// pasteboard. Any attempt to do so returns Prohibited without a secret
/// payload observed by the runtime as copied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentClipboardResult {
    pub action: ClipboardAction,
    pub outcome: ClipboardOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipboardOutcome {
    Prohibited,
    Allowed,
    CleanupPerformed,
}

impl AgentClipboardResult {
    /// Agent copy of a secret is always prohibited.
    pub fn agent_copy_secret_result() -> Self {
        Self {
            action: ClipboardAction::AgentCopySecret,
            outcome: ClipboardOutcome::Prohibited,
        }
    }

    /// User-initiated copy of a permitted secret uses the native capability
    /// with concealed/transient metadata where supported.
    pub fn user_copy_secret_result() -> Self {
        Self {
            action: ClipboardAction::UserCopySecret,
            outcome: ClipboardOutcome::Allowed,
        }
    }
}

/// Metadata to apply to a user-initiated clipboard item that carries a
/// permitted secret, where the platform supports it.
///
/// We use:
///   - org.nspasteboard.ConcealedType
///   - org.nspasteboard.TransientType
///
/// These are best-effort clipboard hygiene. They do NOT guarantee Universal
/// Clipboard will never transfer the value. The strict security invariant is
/// enforced on the agent side (agent secret copy prohibited), not on the
/// system pasteboard.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItemMetadata {
    #[serde(rename = "concealed")]
    pub concealed: bool,
    #[serde(rename = "transient")]
    pub transient: bool,
    pub expiry_monotonic: Option<u64>,
}

impl ClipboardItemMetadata {
    pub fn concealed_transient(expiry_monotonic: Option<u64>) -> Self {
        Self {
            concealed: true,
            transient: true,
            expiry_monotonic,
        }
    }
}

/// The runtime represents clipboard state to the frontend only via these
/// non-secret descriptors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardDescriptor {
    pub last_user_copy_action: Option<ClipboardAction>,
    pub cleared: bool,
    /// Whether a secret was present in the clipboard at last user-copy time
    /// (for display only; actual secret content is never carried).
    pub had_secret: bool,
}

impl ClipboardDescriptor {
    pub fn after_user_secret_copy() -> Self {
        Self {
            last_user_copy_action: Some(ClipboardAction::UserCopySecret),
            cleared: false,
            had_secret: true,
        }
    }

    pub fn after_cleanup() -> Self {
        Self {
            last_user_copy_action: None,
            cleared: true,
            had_secret: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_secret_copy_is_prohibited() {
        let r = AgentClipboardResult::agent_copy_secret_result();
        assert_eq!(r.outcome, ClipboardOutcome::Prohibited);
        assert_eq!(r.action, ClipboardAction::AgentCopySecret);
    }

    #[test]
    fn user_secret_copy_is_allowed_via_native_path() {
        let r = AgentClipboardResult::user_copy_secret_result();
        assert_eq!(r.outcome, ClipboardOutcome::Allowed);
        assert_eq!(r.action, ClipboardAction::UserCopySecret);
    }

    #[test]
    fn clipboard_descriptor_after_cleanup_has_no_secret() {
        let d = ClipboardDescriptor::after_cleanup();
        assert!(d.cleared);
        assert!(!d.had_secret);
    }
}
