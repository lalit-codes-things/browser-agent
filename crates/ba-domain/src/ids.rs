use core::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Maximum accepted length of an opaque identifier, in bytes.
pub const MAX_ID_LEN: usize = 64;

/// Errors produced while validating an opaque identifier against hostile input.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdError {
    /// The identifier was empty.
    #[error("identifier must not be empty")]
    Empty,
    /// The identifier exceeded [`MAX_ID_LEN`].
    #[error("identifier must be at most {MAX_ID_LEN} bytes")]
    TooLong,
    /// The identifier contained a character outside the accepted set.
    #[error("identifier contains a character outside [A-Za-z0-9._:-]")]
    InvalidCharacter,
}

/// Returns whether `value` is a syntactically valid opaque identifier.
#[must_use]
pub fn is_valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ID_LEN
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b':' | b'-'))
}

/// An opaque, validated, non-secret identifier.
///
/// Identifiers are printed in diagnostics and audit metadata, so the accepted
/// alphabet is deliberately narrow and refuses anything that could smuggle
/// whitespace, path separators or control characters into a log line.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpaqueId(String);

impl OpaqueId {
    /// Validates and wraps `value`.
    ///
    /// # Errors
    ///
    /// Returns [`IdError`] when `value` is empty, too long or uses a character
    /// outside the accepted set.
    pub fn parse(value: impl Into<String>) -> Result<Self, IdError> {
        let value = value.into();
        if value.is_empty() {
            return Err(IdError::Empty);
        }
        if value.len() > MAX_ID_LEN {
            return Err(IdError::TooLong);
        }
        if !is_valid_id(&value) {
            return Err(IdError::InvalidCharacter);
        }
        Ok(Self(value))
    }

    /// Builds an identifier from a prefix and a number.
    ///
    /// The prefix is filtered to the accepted alphabet and truncated, so the
    /// result is always valid and this constructor is infallible. It exists for
    /// runtime-owned identifiers only; boundary input must go through
    /// [`OpaqueId::parse`] so malformed values are rejected instead of repaired.
    #[must_use]
    pub fn from_number(prefix: &str, number: impl fmt::Display) -> Self {
        let mut value: String = prefix
            .chars()
            .filter(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | ':' | '-')
            })
            .take(40)
            .collect();
        if value.is_empty() {
            value.push('i');
        }
        value.push('-');
        value.push_str(&number.to_string());
        value.truncate(MAX_ID_LEN);
        Self(value)
    }

    /// Borrows the identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OpaqueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for OpaqueId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for OpaqueId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(raw).map_err(serde::de::Error::custom)
    }
}

macro_rules! opaque_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(OpaqueId);

        impl $name {
            /// Validates and wraps `value`.
            ///
            /// # Errors
            ///
            /// Returns [`IdError`] for malformed input.
            pub fn parse(value: impl Into<String>) -> Result<Self, IdError> {
                OpaqueId::parse(value).map(Self)
            }

            /// Builds an identifier from a prefix and a number.
            #[must_use]
            pub fn from_number(prefix: &str, number: impl core::fmt::Display) -> Self {
                Self(OpaqueId::from_number(prefix, number))
            }

            /// Borrows the identifier.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.0.as_str())
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.0.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                OpaqueId::deserialize(deserializer)
                    .map(Self)
                    .map_err(serde::de::Error::custom)
            }
        }
    };
}

opaque_id! {
    /// Identity of one Agent task.
    TaskId
}
opaque_id! {
    /// Identity of one proposed action.
    ActionId
}
opaque_id! {
    /// Identity of one browser tab.
    TabId
}
opaque_id! {
    /// Identity of one browser frame inside a tab.
    FrameId
}
opaque_id! {
    /// Identity of one grounded element inside an observation.
    NodeId
}
opaque_id! {
    /// Identity of one native application window.
    WindowId
}
opaque_id! {
    /// Identity of one observation epoch.
    ObservationId
}
opaque_id! {
    /// Pseudonymous identity of one CEF request context.
    BrowserContextId
}
opaque_id! {
    /// Opaque reference to a vault entry. Never carries a secret value.
    CredentialRef
}
opaque_id! {
    /// Identity of a stored, reviewed skill version.
    SkillId
}
opaque_id! {
    /// Identity of a local research note.
    NoteId
}
opaque_id! {
    /// Identity of one audit segment.
    AuditSegmentId
}

/// Monotonic event sequence assigned by the trusted runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Sequence(u64);

impl Sequence {
    /// Wraps `value`.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Returns the next sequence value, or `None` on overflow.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

impl fmt::Display for Sequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Monotonic task revision, incremented on every accepted transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Revision(u64);

impl Revision {
    /// Wraps `value`.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Returns the next revision, or `None` on overflow.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

impl fmt::Display for Revision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_expected_identifiers() {
        assert!(TaskId::parse("task-0001").is_ok());
        assert!(FrameId::parse("frame:0:main").is_ok());
    }

    #[test]
    fn rejects_hostile_identifiers() {
        assert_eq!(TaskId::parse(""), Err(IdError::Empty));
        assert_eq!(
            TaskId::parse("a".repeat(MAX_ID_LEN + 1)),
            Err(IdError::TooLong)
        );
        assert_eq!(
            TaskId::parse("../etc/passwd"),
            Err(IdError::InvalidCharacter)
        );
        assert_eq!(TaskId::parse("task 1"), Err(IdError::InvalidCharacter));
    }

    #[test]
    fn derived_identifiers_are_always_valid() {
        assert_eq!(TaskId::from_number("task", 7).as_str(), "task-7");
        assert_eq!(
            TaskId::from_number("seg/ment with spaces", 3).as_str(),
            "segmentwithspaces-3"
        );
        assert!(!TaskId::from_number("", 1).as_str().is_empty());
        assert!(is_valid_id(TaskId::from_number("task", u64::MAX).as_str()));
    }

    #[test]
    fn sequence_overflow_is_explicit() {
        assert_eq!(Sequence::new(u64::MAX).next(), None);
        assert_eq!(Sequence::new(1).next(), Some(Sequence::new(2)));
    }
}
