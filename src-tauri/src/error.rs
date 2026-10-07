use serde::Serialize;
use thiserror::Error;

/// Application-level error enumeration.
///
/// We prefer explicit typed errors over stringly-typed panic paths
/// (C-60: no-panic/no-unwrap policy).
#[derive(Debug, Error)]
pub enum Error {
    #[error("operation not implemented yet: {0}")]
    NotImplemented(String),

    #[error("operation aborted: {0}")]
    Aborted(String),

    #[error("unsupported action: {0}")]
    Unsupported(String),

    #[error("policy blocked action: {0}")]
    PolicyBlocked(String),

    #[error("verification inconclusive: {0}")]
    VerificationUnknown(String),

    #[error("state mismatch: {0}")]
    StateMismatch(String),

    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().trim().as_ref())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(value: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Error::Internal(value.to_string())
    }
}
