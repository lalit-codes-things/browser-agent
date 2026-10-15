//! The typed action grammar (Section 12.3).
//!
//! The model may only produce a schema-constrained typed action. There is no
//! JavaScript, shell, CDP, filesystem or generic tool invocation path in this
//! vocabulary, and unknown action types or unexpected fields are rejected.
//!
//! Structural deserialization and bound enforcement are deliberately separate:
//! [`Action`] is the wire shape, and [`ValidatedAction`] is the only form the
//! policy engine and executor accept. Deserializing a `ValidatedAction` runs
//! [`ValidatedAction::new`], so an IPC payload cannot reach policy without its
//! bounds being checked.

use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ids::{ActionId, CredentialRef, FrameId, NodeId, ObservationId, TabId};
use crate::origin::Origin;

/// Maximum code points accepted in a navigation URL.
pub const MAX_NAVIGATION_URL_CHARS: usize = 2_048;
/// Maximum code points accepted in a typed value.
pub const MAX_TYPED_TEXT_CHARS: usize = 4_000;
/// Maximum code points accepted in a selected option value.
pub const MAX_OPTION_CHARS: usize = 512;
/// Maximum code points accepted in a handoff explanation.
pub const MAX_HANDOFF_REASON_CHARS: usize = 512;
/// Maximum code points accepted in any bounded text field.
pub const MAX_ANY_TEXT_CHARS: usize = crate::MAX_TASK_TEXT_CODEPOINTS;

/// A rejected text field.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TextBoundError {
    /// The value exceeded the field's code-point limit.
    #[error("`{field}` exceeds {limit} code points (got {actual})")]
    TooLong {
        /// Field name.
        field: &'static str,
        /// Limit in code points.
        limit: usize,
        /// Measured length in code points.
        actual: usize,
    },
    /// The value contained a null byte or a control character that cannot be
    /// displayed as text.
    #[error("`{field}` contains a disallowed control character")]
    ControlCharacter {
        /// Field name.
        field: &'static str,
    },
    /// A numeric field was outside its accepted range.
    #[error("`{field}` is outside its accepted range")]
    OutOfRange {
        /// Field name.
        field: &'static str,
    },
}

/// Text that is bounded and free of hidden control characters.
///
/// On deserialization the generous [`MAX_ANY_TEXT_CHARS`] bound is applied;
/// [`ValidatedAction::new`] then applies each field's tighter bound.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundedText(String);

impl BoundedText {
    /// Validates `value` against `limit` code points.
    ///
    /// # Errors
    ///
    /// Returns [`TextBoundError`] when the bound or the character class fails.
    pub fn parse(
        value: impl Into<String>,
        field: &'static str,
        limit: usize,
    ) -> Result<Self, TextBoundError> {
        let value = value.into();
        let actual = value.chars().count();
        if actual > limit {
            return Err(TextBoundError::TooLong {
                field,
                limit,
                actual,
            });
        }
        if value
            .chars()
            .any(|c| c == '\0' || (c.is_control() && c != '\n' && c != '\t'))
        {
            return Err(TextBoundError::ControlCharacter { field });
        }
        Ok(Self(value))
    }

    /// Borrows the value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for BoundedText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for BoundedText {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for BoundedText {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(raw, "text", MAX_ANY_TEXT_CHARS).map_err(serde::de::Error::custom)
    }
}

/// Action verb of the typed grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionKind {
    /// Activate a grounded element.
    Click,
    /// Enter text into a grounded field.
    Type,
    /// Choose an option in a grounded control.
    Select,
    /// Top-level navigation to a URL.
    Navigate,
    /// Scroll the active page.
    Scroll,
    /// Bounded stabilization wait.
    Wait,
    /// Trusted secure-fill of an approved credential reference.
    SecureFill,
    /// Structured request for approval of one bounded action.
    RequestConfirmation,
    /// Stop dispatch and hand browser control to the user.
    HandoffToUser,
}

impl ActionKind {
    /// Returns the canonical wire code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Click => "CLICK",
            Self::Type => "TYPE",
            Self::Select => "SELECT",
            Self::Navigate => "NAVIGATE",
            Self::Scroll => "SCROLL",
            Self::Wait => "WAIT",
            Self::SecureFill => "SECURE_FILL",
            Self::RequestConfirmation => "REQUEST_CONFIRMATION",
            Self::HandoffToUser => "HANDOFF_TO_USER",
        }
    }
}

impl fmt::Display for ActionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// Scroll direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScrollDirection {
    /// Toward the top of the document.
    Up,
    /// Toward the bottom of the document.
    Down,
    /// Toward the start of the line.
    Left,
    /// Toward the end of the line.
    Right,
}

/// A grounded target inside one observation epoch.
///
/// `role` and `label` originate from page-controlled content and remain
/// untrusted data: they are display hints for confirmation copy and never
/// authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetRef {
    /// Grounded node identity.
    pub node: NodeId,
    /// Frame that owns the node.
    pub frame: FrameId,
    /// Tab that owns the frame.
    pub tab: TabId,
    /// Exact origin of the frame.
    pub origin: Origin,
    /// Observation epoch the node identity is scoped to.
    pub epoch: ObservationId,
    /// Accessible role hint, untrusted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<BoundedText>,
    /// Accessible name hint, untrusted and already redacted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<BoundedText>,
}

impl TargetRef {
    /// Returns whether `other` identifies the same element in the same epoch.
    #[must_use]
    pub fn is_same_grounding(&self, other: &Self) -> bool {
        self.node == other.node
            && self.frame == other.frame
            && self.tab == other.tab
            && self.epoch == other.epoch
            && self.origin == other.origin
    }
}

/// Parameters for `CLICK`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClickParams {
    /// Grounded target.
    pub target: TargetRef,
    /// Whether the action opens a new tab or window.
    #[serde(default)]
    pub opens_new_tab: bool,
    /// Whether the action may submit a form.
    #[serde(default)]
    pub may_submit: bool,
}

/// Parameters for `TYPE`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeParams {
    /// Grounded field.
    pub target: TargetRef,
    /// Value to enter. Never a raw vault secret.
    pub text: BoundedText,
    /// Whether the runtime should press Enter after typing.
    #[serde(default)]
    pub submit: bool,
}

/// Parameters for `SELECT`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectParams {
    /// Grounded control.
    pub target: TargetRef,
    /// Option value or visible label.
    pub option: BoundedText,
}

/// Parameters for `NAVIGATE`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigateParams {
    /// Absolute destination URL.
    pub url: BoundedText,
}

/// Parameters for `SCROLL`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScrollParams {
    /// Direction.
    pub direction: ScrollDirection,
    /// Number of viewport steps, 1..=100.
    pub amount: u16,
}

/// Parameters for `WAIT`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaitParams {
    /// Maximum wait in milliseconds, 1..=30000.
    pub max_ms: u32,
}

/// Parameters for `SECURE_FILL`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecureFillParams {
    /// Grounded field. Its origin is the only origin eligible for the value.
    pub target: TargetRef,
    /// Opaque vault reference; never a secret value.
    pub credential: CredentialRef,
}

/// Parameters for `REQUEST_CONFIRMATION`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfirmationParams {
    /// The action awaiting approval.
    pub action_id: ActionId,
}

/// Parameters for `HANDOFF_TO_USER`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffParams {
    /// Grounded target the user must act on.
    pub target: TargetRef,
    /// Plain-language reason shown to the user.
    pub reason: BoundedText,
}

/// A structurally parsed typed action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "params", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Action {
    /// Activate a grounded element.
    Click(ClickParams),
    /// Enter text into a grounded field.
    Type(TypeParams),
    /// Choose an option.
    Select(SelectParams),
    /// Navigate to a URL.
    Navigate(NavigateParams),
    /// Scroll the page.
    Scroll(ScrollParams),
    /// Bounded wait.
    Wait(WaitParams),
    /// Context-restricted secure fill.
    SecureFill(SecureFillParams),
    /// Request approval of another action.
    RequestConfirmation(ConfirmationParams),
    /// Hand control to the user.
    HandoffToUser(HandoffParams),
}

/// A digest binding an authorization to the exact action parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionDigest([u8; 32]);

impl ActionDigest {
    /// Returns the raw bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Returns the lowercase hex encoding.
    #[must_use]
    pub fn to_hex(self) -> String {
        let mut out = String::with_capacity(64);
        for byte in self.0 {
            out.push_str(&format!("{byte:02x}"));
        }
        out
    }
}

impl fmt::Display for ActionDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl FromStr for ActionDigest {
    type Err = TextBoundError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != 64 {
            return Err(TextBoundError::OutOfRange { field: "digest" });
        }
        let mut bytes = [0u8; 32];
        let (pairs, _remainder) = value.as_bytes().as_chunks::<2>();
        for (index, chunk) in pairs.iter().enumerate() {
            let text = core::str::from_utf8(chunk)
                .map_err(|_| TextBoundError::OutOfRange { field: "digest" })?;
            bytes[index] = u8::from_str_radix(text, 16)
                .map_err(|_| TextBoundError::OutOfRange { field: "digest" })?;
        }
        Ok(Self(bytes))
    }
}

impl Serialize for ActionDigest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for ActionDigest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::from_str(&raw).map_err(serde::de::Error::custom)
    }
}

impl Action {
    /// Returns the action verb.
    #[must_use]
    pub const fn kind(&self) -> ActionKind {
        match self {
            Self::Click(_) => ActionKind::Click,
            Self::Type(_) => ActionKind::Type,
            Self::Select(_) => ActionKind::Select,
            Self::Navigate(_) => ActionKind::Navigate,
            Self::Scroll(_) => ActionKind::Scroll,
            Self::Wait(_) => ActionKind::Wait,
            Self::SecureFill(_) => ActionKind::SecureFill,
            Self::RequestConfirmation(_) => ActionKind::RequestConfirmation,
            Self::HandoffToUser(_) => ActionKind::HandoffToUser,
        }
    }

    /// Returns the grounded target, when the action has one.
    #[must_use]
    pub fn target(&self) -> Option<&TargetRef> {
        match self {
            Self::Click(params) => Some(&params.target),
            Self::Type(params) => Some(&params.target),
            Self::Select(params) => Some(&params.target),
            Self::SecureFill(params) => Some(&params.target),
            Self::HandoffToUser(params) => Some(&params.target),
            Self::Navigate(_) | Self::Scroll(_) | Self::Wait(_) | Self::RequestConfirmation(_) => {
                None
            }
        }
    }

    /// Returns whether this action initiates a top-level navigation and so
    /// consumes navigation budget (BA-TASK-BUDGET-001).
    #[must_use]
    pub fn is_agent_initiated_navigation(&self) -> bool {
        matches!(self, Self::Navigate(_))
    }

    /// Returns the destination origin when the action names one explicitly.
    #[must_use]
    pub fn destination_origin(&self) -> Option<Origin> {
        match self {
            Self::Navigate(params) => Origin::parse(params.url.as_str()).ok(),
            other => other.target().map(|target| target.origin.clone()),
        }
    }

    /// Returns the digest that binds authorizations and detects parameter
    /// mutation (Section 13.3).
    #[must_use]
    pub fn digest(&self) -> ActionDigest {
        digest_of(self)
    }

    /// Returns a digest over verb and grounded target only.
    ///
    /// Used by the no-progress rule (BA-STUCK-001): repeating an identical
    /// verb against an identical target is not progress, even if typed text
    /// differs.
    #[must_use]
    pub fn normalized_digest(&self) -> ActionDigest {
        let material = match self {
            Self::Click(params) => format!("CLICK|{}", grounding_key(&params.target)),
            Self::Type(params) => format!("TYPE|{}", grounding_key(&params.target)),
            Self::Select(params) => format!("SELECT|{}", grounding_key(&params.target)),
            Self::Navigate(params) => {
                let origin = Origin::parse(params.url.as_str())
                    .map_or_else(|_| "invalid".to_owned(), |o| o.canonical());
                format!("NAVIGATE|{origin}")
            }
            Self::Scroll(params) => format!("SCROLL|{:?}", params.direction),
            Self::Wait(_) => "WAIT".to_owned(),
            Self::SecureFill(params) => format!("SECURE_FILL|{}", grounding_key(&params.target)),
            Self::RequestConfirmation(params) => {
                format!("REQUEST_CONFIRMATION|{}", params.action_id)
            }
            Self::HandoffToUser(params) => format!("HANDOFF|{}", grounding_key(&params.target)),
        };
        ActionDigest(Sha256::digest(material.as_bytes()).into())
    }
}

/// An action whose per-field bounds have been checked.
///
/// This is the only action form accepted by policy and the executor. The wire
/// form is [`Action`]; the conversion is infallible in the other direction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ValidatedAction(Action);

impl ValidatedAction {
    /// Validates per-field bounds.
    ///
    /// # Errors
    ///
    /// Returns [`TextBoundError`] when a field exceeds its schema bound or a
    /// numeric field is out of range.
    pub fn new(action: Action) -> Result<Self, TextBoundError> {
        match &action {
            Action::Click(params) => {
                check_target(&params.target)?;
            }
            Action::Type(params) => {
                check_target(&params.target)?;
                BoundedText::parse(params.text.as_str(), "type.text", MAX_TYPED_TEXT_CHARS)?;
            }
            Action::Select(params) => {
                check_target(&params.target)?;
                BoundedText::parse(params.option.as_str(), "select.option", MAX_OPTION_CHARS)?;
            }
            Action::Navigate(params) => {
                BoundedText::parse(
                    params.url.as_str(),
                    "navigate.url",
                    MAX_NAVIGATION_URL_CHARS,
                )?;
            }
            Action::Scroll(params) => {
                if params.amount == 0 || params.amount > 100 {
                    return Err(TextBoundError::OutOfRange {
                        field: "scroll.amount",
                    });
                }
            }
            Action::Wait(params) => {
                if params.max_ms == 0 || params.max_ms > 30_000 {
                    return Err(TextBoundError::OutOfRange {
                        field: "wait.max_ms",
                    });
                }
            }
            Action::SecureFill(params) => {
                check_target(&params.target)?;
            }
            Action::RequestConfirmation(_) | Action::HandoffToUser(_) => {}
        }
        Ok(Self(action))
    }

    /// Borrows the validated action.
    #[must_use]
    pub const fn action(&self) -> &Action {
        &self.0
    }

    /// Consumes the wrapper, returning the wire form.
    #[must_use]
    pub fn into_inner(self) -> Action {
        self.0
    }

    /// Returns the action digest.
    #[must_use]
    pub fn digest(&self) -> ActionDigest {
        self.0.digest()
    }

    /// Returns the normalized no-progress digest.
    #[must_use]
    pub fn normalized_digest(&self) -> ActionDigest {
        self.0.normalized_digest()
    }

    /// Returns the action verb.
    #[must_use]
    pub const fn kind(&self) -> ActionKind {
        self.0.kind()
    }
}

impl<'de> Deserialize<'de> for ValidatedAction {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let action = Action::deserialize(deserializer)?;
        Self::new(action).map_err(serde::de::Error::custom)
    }
}

fn check_target(target: &TargetRef) -> Result<(), TextBoundError> {
    if let Some(role) = &target.role {
        BoundedText::parse(role.as_str(), "target.role", 128)?;
    }
    if let Some(label) = &target.label {
        BoundedText::parse(label.as_str(), "target.label", 512)?;
    }
    Ok(())
}

fn grounding_key(target: &TargetRef) -> String {
    format!(
        "{}/{}/{}/{}",
        target.origin.canonical(),
        target.tab,
        target.frame,
        target.node
    )
}

fn digest_of<T: Serialize>(value: &T) -> ActionDigest {
    match serde_json::to_vec(value) {
        Ok(bytes) => ActionDigest(Sha256::digest(&bytes).into()),
        Err(_) => ActionDigest([0u8; 32]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{FrameId, NodeId, ObservationId, TabId};

    fn target(node: &str) -> TargetRef {
        TargetRef {
            node: NodeId::parse(node).unwrap(),
            frame: FrameId::parse("frame-0").unwrap(),
            tab: TabId::parse("tab-0").unwrap(),
            origin: Origin::parse("https://shop.example").unwrap(),
            epoch: ObservationId::parse("epoch-3").unwrap(),
            role: Some(BoundedText::parse("button", "target.role", 128).unwrap()),
            label: Some(BoundedText::parse("Add to cart", "target.label", 512).unwrap()),
        }
    }

    #[test]
    fn rejects_unknown_action_kinds() {
        let raw = r#"{"kind":"EXECUTE_SHELL","params":{"cmd":"rm -rf /"}}"#;
        assert!(serde_json::from_str::<Action>(raw).is_err());
    }

    #[test]
    fn rejects_authority_bearing_extra_fields() {
        let raw = r#"{"kind":"NAVIGATE","params":{"url":"https://example.com","authorized":true}}"#;
        assert!(serde_json::from_str::<Action>(raw).is_err());
    }

    #[test]
    fn validated_action_enforces_field_bounds() {
        let too_long = "a".repeat(MAX_TYPED_TEXT_CHARS + 1);
        let action = Action::Type(TypeParams {
            target: target("node-1"),
            text: BoundedText::parse(too_long, "type.text", MAX_ANY_TEXT_CHARS).unwrap(),
            submit: false,
        });
        assert!(ValidatedAction::new(action).is_err());

        let ok = Action::Type(TypeParams {
            target: target("node-1"),
            text: BoundedText::parse("hello", "type.text", MAX_ANY_TEXT_CHARS).unwrap(),
            submit: false,
        });
        assert!(ValidatedAction::new(ok).is_ok());
    }

    #[test]
    fn validated_action_rejects_out_of_range_numbers() {
        let action = Action::Scroll(ScrollParams {
            direction: ScrollDirection::Down,
            amount: 0,
        });
        assert!(matches!(
            ValidatedAction::new(action),
            Err(TextBoundError::OutOfRange { .. })
        ));
        let action = Action::Wait(WaitParams { max_ms: 90_000 });
        assert!(matches!(
            ValidatedAction::new(action),
            Err(TextBoundError::OutOfRange { .. })
        ));
    }

    #[test]
    fn deserializing_a_validated_action_runs_the_bounds_check() {
        let raw = r#"{"kind":"WAIT","params":{"max_ms":99000}}"#;
        assert!(serde_json::from_str::<ValidatedAction>(raw).is_err());
    }

    #[test]
    fn digest_changes_when_parameters_change() {
        let a = Action::Click(ClickParams {
            target: target("node-1"),
            opens_new_tab: false,
            may_submit: false,
        });
        let b = Action::Click(ClickParams {
            target: target("node-2"),
            opens_new_tab: false,
            may_submit: false,
        });
        assert_ne!(a.digest(), b.digest());
        assert_eq!(a.digest(), a.clone().digest());
    }

    #[test]
    fn normalized_digest_ignores_typed_text_but_keeps_grounding() {
        let make = |text: &str| {
            Action::Type(TypeParams {
                target: target("node-1"),
                text: BoundedText::parse(text, "type.text", MAX_ANY_TEXT_CHARS).unwrap(),
                submit: false,
            })
        };
        assert_eq!(
            make("one").normalized_digest(),
            make("two").normalized_digest()
        );
        assert_ne!(
            make("one").normalized_digest(),
            Action::Type(TypeParams {
                target: target("node-9"),
                text: BoundedText::parse("one", "type.text", MAX_ANY_TEXT_CHARS).unwrap(),
                submit: false,
            })
            .normalized_digest()
        );
    }

    #[test]
    fn control_characters_are_rejected() {
        assert!(BoundedText::parse("bad\u{7}value", "type.text", 100).is_err());
        assert!(BoundedText::parse("line\nbreak", "type.text", 100).is_ok());
    }

    #[test]
    fn digest_hex_round_trips() {
        let digest = Action::Navigate(NavigateParams {
            url: BoundedText::parse("https://example.com", "navigate.url", 2_048).unwrap(),
        })
        .digest();
        let hex = digest.to_hex();
        assert_eq!(hex.len(), 64);
        assert_eq!(ActionDigest::from_str(&hex).unwrap(), digest);
        assert!(ActionDigest::from_str("not-a-digest").is_err());
    }
}
