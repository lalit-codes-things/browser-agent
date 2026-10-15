//! Browser-Agent shared domain schema.
//!
//! This crate is the single definition point for the vocabulary shared by the
//! Rust trusted core, the IPC boundary and the first-party UI: identifiers,
//! exact origins, task lifecycle states, wait reasons, the typed action
//! grammar, the verification vocabulary and the canonical user-facing copy
//! tables.
//!
//! The crate is deliberately passive. It holds no policy decisions, no
//! authority and no runtime state, so it can be depended on by every other
//! crate without creating a second authority.
//!
//! Requirement references: BA-STATE-001, BA-STATE-002, BA-READY-001,
//! BA-VERIFY-001, BA-TRANSPARENCY-001.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![deny(missing_docs)]

mod action;
mod copy;
mod error;
mod errorcopy;
mod event;
mod ids;
mod origin;
mod redact;
mod task;

pub use action::{
    Action, ActionDigest, ActionKind, BoundedText, ClickParams, ConfirmationParams, HandoffParams,
    MAX_ANY_TEXT_CHARS, MAX_HANDOFF_REASON_CHARS, MAX_NAVIGATION_URL_CHARS, MAX_OPTION_CHARS,
    MAX_TYPED_TEXT_CHARS, NavigateParams, ScrollDirection, ScrollParams, SecureFillParams,
    SelectParams, TargetRef, TextBoundError, TypeParams, ValidatedAction, WaitParams,
};
pub use copy::{
    CopyTableError, STUCK_CONTROLS, STUCK_LABEL, case_name, copy_table_json, readiness_controls,
    readiness_label, task_state_label, verification_label, verification_meaning,
    wait_reason_controls, wait_reason_label,
};
pub use error::DomainError;
pub use errorcopy::{ErrorCase, FourPartError};
pub use event::{
    BudgetField, ClockError, Envelope, EventValidationError, TaskBudgetView, TaskEvent, UnixMillis,
    canonical_json,
};
pub use ids::{
    ActionId, AuditSegmentId, BrowserContextId, CredentialRef, FrameId, IdError, MAX_ID_LEN,
    NodeId, NoteId, ObservationId, OpaqueId, Revision, Sequence, SkillId, TabId, TaskId, WindowId,
    is_valid_id,
};
pub use origin::{Origin, OriginError, OriginScheme, url_discloses_credentials};
pub use redact::{REDACTED, is_secret_like, mask_secret_like, redact_url};
pub use task::{
    ActionClass, ProfileKind, ReadinessState, TaskState, TaskStateError, VerificationOutcome,
    WaitReason,
};

/// Wire schema version for every Browser-Agent boundary (Section 20.2).
///
/// Bump only together with a reviewed contract change; every consumer validates
/// this value at runtime and fails closed on a mismatch.
pub const SCHEMA_VERSION: u16 = 1;

/// Product identifier used to namespace local data directories and IPC frames.
pub const PRODUCT: &str = "browser-agent";

/// Origin of the first-party UI served through the registered CEF scheme.
pub const UI_ORIGIN: &str = "browser-agent://ui";

/// Hard cap for a single decoded IPC frame, applied before deserialization.
pub const MAX_IPC_FRAME_BYTES: usize = 512 * 1024;

/// Maximum accepted composer length in Unicode code points (BA-COMPOSER-001).
pub const MAX_TASK_TEXT_CODEPOINTS: usize = 8_000;
