//! Typed domain errors.
//!
//! Every variant is fail-closed: callers may not convert one of these into an
//! allow-shaped default (Section 20.6).

/// Errors raised by the shared domain schema.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    /// An identifier failed validation.
    #[error("invalid identifier: {0}")]
    Id(#[from] crate::ids::IdError),
    /// An origin failed validation.
    #[error("invalid origin: {0}")]
    Origin(#[from] crate::origin::OriginError),
    /// A task event violated the lifecycle contract.
    #[error("invalid task event: {0}")]
    Event(#[from] crate::event::EventValidationError),
    /// A bounded text field exceeded its schema limit.
    #[error("text bound violated: {0}")]
    Text(#[from] crate::action::TextBoundError),
    /// The canonical copy table was requested for an unknown state.
    #[error("canonical copy unavailable: {0}")]
    Copy(#[from] crate::copy::CopyTableError),
    /// A task state transition is not allowed by the transition table.
    #[error("illegal task transition: {0}")]
    Transition(#[from] crate::task::TaskStateError),
}
