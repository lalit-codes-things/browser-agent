use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct PublicError {
    pub code: &'static str,
    pub message_key: &'static str,
}

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
        let public = match self {
            Self::NotImplemented(_) => PublicError {
                code: "NOT_IMPLEMENTED",
                message_key: "error.not_implemented",
            },
            Self::Aborted(_) => PublicError {
                code: "ABORTED",
                message_key: "error.aborted",
            },
            Self::Unsupported(_) => PublicError {
                code: "UNSUPPORTED",
                message_key: "error.unsupported",
            },
            Self::PolicyBlocked(_) => PublicError {
                code: "POLICY_BLOCKED",
                message_key: "error.policy_blocked",
            },
            Self::VerificationUnknown(_) => PublicError {
                code: "VERIFICATION_UNKNOWN",
                message_key: "error.verification_unknown",
            },
            Self::StateMismatch(_) => PublicError {
                code: "STATE_MISMATCH",
                message_key: "error.state_mismatch",
            },
            Self::InvalidParameter(_) => PublicError {
                code: "INVALID_PARAMETER",
                message_key: "error.invalid_parameter",
            },
            Self::Internal(_) => PublicError {
                code: "INTERNAL",
                message_key: "error.internal",
            },
        };
        public.serialize(serializer)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(value: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Error::Internal(value.to_string())
    }
}
