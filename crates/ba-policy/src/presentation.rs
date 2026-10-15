//! Structured presentation of decisions (BA-APPROVAL-001, Section 13.3).
//!
//! Every confirmation and handoff screen is generated from these trusted
//! structure fields, never from model prose. Parameter values are redacted
//! before they are placed here, and the presentation never carries a secret
//! value, only opaque references.

use ba_domain::{Action, ActionClass, ActionKind, TargetRef, mask_secret_like, redact_url};

use crate::PolicyRequest;
use crate::rules::RuleId;

/// One non-secret parameter shown to the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonSecretParameter {
    /// Parameter label in plain language.
    pub name: String,
    /// Redacted value.
    pub value: String,
}

/// Structured description of the action awaiting approval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPresentation {
    /// Action verb.
    pub action_kind: ActionKind,
    /// Plain-language verb label.
    pub action_label: String,
    /// Exact origin the action happens on.
    pub origin: String,
    /// Destination origin or URL, when the action targets one.
    pub destination: Option<String>,
    /// Plain-language description of the target element.
    pub target_label: Option<String>,
    /// Non-secret parameters.
    pub parameters: Vec<NonSecretParameter>,
    /// Why approval is required, from the canonical rule text.
    pub why: String,
    /// Which rule required it.
    pub rule: RuleId,
    /// Trusted consequence class.
    pub class: ActionClass,
    /// What the user should expect to happen.
    pub expected_effect: String,
    /// What Undo will attempt, when a tested compensation exists.
    pub undo_description: Option<String>,
}

/// Structured description of an action the user must perform themselves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffPresentation {
    /// Action verb the user is being asked to perform.
    pub action_kind: ActionKind,
    /// Plain-language verb label.
    pub action_label: String,
    /// Exact origin the action happens on.
    pub origin: String,
    /// Destination origin or URL, when the action targets one.
    pub destination: Option<String>,
    /// Plain-language description of the target element.
    pub target_label: Option<String>,
    /// Non-secret parameters.
    pub parameters: Vec<NonSecretParameter>,
    /// Known consequences of performing the action.
    pub consequences: String,
    /// Why the agent stopped.
    pub reason: String,
    /// Which rule required the handoff.
    pub rule: RuleId,
    /// Trusted consequence class.
    pub class: ActionClass,
    /// What the user should do next.
    pub user_action_hint: String,
}

/// Builds the presentation for an approvable action.
#[must_use]
pub fn for_action(
    request: &PolicyRequest<'_>,
    rule: RuleId,
    class: ActionClass,
) -> ActionPresentation {
    let action = request.action.action();
    let (destination, parameters) = parameters_of(action);
    ActionPresentation {
        action_kind: action.kind(),
        action_label: action_label(action.kind()).to_owned(),
        origin: request.context.origin.canonical(),
        destination,
        target_label: action.target().map(describe_target),
        parameters,
        why: rule.plain_language().to_owned(),
        rule,
        class,
        expected_effect: expected_effect(action.kind()).to_owned(),
        undo_description: request.capability.map(|capability| {
            format!(
                "Undo runs the tested operation `{}`, which should restore: {}",
                capability.compensation_operation, capability.compensation_postcondition
            )
        }),
    }
}

/// Builds the presentation for an action the user must perform.
#[must_use]
pub fn for_handoff(
    request: &PolicyRequest<'_>,
    rule: RuleId,
    class: ActionClass,
) -> HandoffPresentation {
    let action = request.action.action();
    let (destination, parameters) = parameters_of(action);
    HandoffPresentation {
        action_kind: action.kind(),
        action_label: action_label(action.kind()).to_owned(),
        origin: request.context.origin.canonical(),
        destination,
        target_label: action.target().map(describe_target),
        parameters,
        consequences: consequences_of(class).to_owned(),
        reason: rule.plain_language().to_owned(),
        rule,
        class,
        user_action_hint:
            "Take over the page, perform this step yourself, and resume the task only when you want the agent to continue."
                .to_owned(),
    }
}

/// Returns the plain-language label for an action verb.
#[must_use]
pub const fn action_label(kind: ActionKind) -> &'static str {
    match kind {
        ActionKind::Click => "Click a control on the page",
        ActionKind::Type => "Enter text in a field",
        ActionKind::Select => "Choose an option",
        ActionKind::Navigate => "Open a page",
        ActionKind::Scroll => "Scroll the page",
        ActionKind::Wait => "Wait for the page to settle",
        ActionKind::SecureFill => "Fill a saved login",
        ActionKind::RequestConfirmation => "Ask you to approve an action",
        ActionKind::HandoffToUser => "Hand control to you",
    }
}

fn expected_effect(kind: ActionKind) -> &'static str {
    match kind {
        ActionKind::Click => "The control carries out its normal behaviour on this page.",
        ActionKind::Type => "The field contains the text shown above.",
        ActionKind::Select => "The control shows the option shown above.",
        ActionKind::Navigate => "The page opens in the current tab.",
        ActionKind::Scroll => "The page scrolls. No site data is changed.",
        ActionKind::Wait => "The agent waits for the page to settle. No site data is changed.",
        ActionKind::SecureFill => {
            "Your saved login for this site is entered in the field. The website can read the value once it is entered."
        }
        ActionKind::RequestConfirmation => "You are asked to approve one specific action.",
        ActionKind::HandoffToUser => "The agent stops and you control the page.",
    }
}

fn consequences_of(class: ActionClass) -> &'static str {
    match class {
        ActionClass::Read => "Nothing on the site is changed by this step.",
        ActionClass::ReversibleWrite => {
            "The site changes in a way the app has tested undo evidence for."
        }
        ActionClass::Irreversible => {
            "This step has a lasting effect. Browser-Agent cannot undo it, and the Agent is not allowed to perform it."
        }
        ActionClass::Unknown => {
            "The app cannot establish what this step would do, so the Agent is not allowed to perform it."
        }
    }
}

fn parameters_of(action: &Action) -> (Option<String>, Vec<NonSecretParameter>) {
    let mut parameters = Vec::new();
    let mut destination = None;
    match action {
        Action::Navigate(params) => {
            let redacted = redact_url(params.url.as_str());
            destination = Some(redacted.clone());
            parameters.push(NonSecretParameter {
                name: "Destination".to_owned(),
                value: redacted,
            });
        }
        Action::Type(params) => {
            parameters.push(NonSecretParameter {
                name: "Text to enter".to_owned(),
                value: mask_secret_like(params.text.as_str()).into_owned(),
            });
            parameters.push(NonSecretParameter {
                name: "Submits the form".to_owned(),
                value: yes_no(params.submit).to_owned(),
            });
        }
        Action::Select(params) => {
            parameters.push(NonSecretParameter {
                name: "Option".to_owned(),
                value: mask_secret_like(params.option.as_str()).into_owned(),
            });
        }
        Action::Click(params) => {
            parameters.push(NonSecretParameter {
                name: "May submit a form".to_owned(),
                value: yes_no(params.may_submit).to_owned(),
            });
            parameters.push(NonSecretParameter {
                name: "Opens a new tab".to_owned(),
                value: yes_no(params.opens_new_tab).to_owned(),
            });
        }
        Action::Scroll(params) => {
            parameters.push(NonSecretParameter {
                name: "Direction".to_owned(),
                value: format!("{:?}", params.direction).to_ascii_lowercase(),
            });
            parameters.push(NonSecretParameter {
                name: "Steps".to_owned(),
                value: params.amount.to_string(),
            });
        }
        Action::Wait(params) => {
            parameters.push(NonSecretParameter {
                name: "Maximum wait".to_owned(),
                value: format!("{} ms", params.max_ms),
            });
        }
        Action::SecureFill(params) => {
            parameters.push(NonSecretParameter {
                name: "Saved login".to_owned(),
                value: params.credential.as_str().to_owned(),
            });
            parameters.push(NonSecretParameter {
                name: "Site".to_owned(),
                value: params.target.origin.canonical(),
            });
        }
        Action::RequestConfirmation(params) => {
            parameters.push(NonSecretParameter {
                name: "Action".to_owned(),
                value: params.action_id.as_str().to_owned(),
            });
        }
        Action::HandoffToUser(params) => {
            parameters.push(NonSecretParameter {
                name: "Why".to_owned(),
                value: mask_secret_like(params.reason.as_str()).into_owned(),
            });
        }
    }
    (destination, parameters)
}

fn describe_target(target: &TargetRef) -> String {
    let role = target
        .role
        .as_ref()
        .map(|role| role.as_str().to_owned())
        .unwrap_or_else(|| "control".to_owned());
    match &target.label {
        Some(label) => format!(
            "{role} \u{201c}{}\u{201d}",
            mask_secret_like(label.as_str())
        ),
        None => role,
    }
}

const fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProfileIdentity;
    use ba_domain::{
        ActionClass, BoundedText, ClickParams, CredentialRef, FrameId, NavigateParams, NodeId,
        ObservationId, Origin, ProfileKind, SecureFillParams, TabId, TaskId, TypeParams,
        ValidatedAction,
    };

    fn target() -> TargetRef {
        TargetRef {
            node: NodeId::parse("node-1").unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
            tab: TabId::parse("tab-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            role: Some(BoundedText::parse("textbox", "target.role", 128).unwrap()),
            label: Some(BoundedText::parse("Card number", "target.label", 512).unwrap()),
        }
    }

    fn context() -> crate::PolicyContext {
        crate::PolicyContext {
            task_id: TaskId::parse("task-1").unwrap(),
            tab: TabId::parse("tab-1").unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            profile: ProfileIdentity {
                context: ba_domain::BrowserContextId::parse("ctx-normal").unwrap(),
                kind: ProfileKind::Normal,
            },
            approved_origins: Vec::new(),
            origin_allowlist_enabled: false,
            audit_healthy: true,
            vault_available: true,
            secure_fill_authorized: true,
        }
    }

    fn present(action: &ValidatedAction) -> HandoffPresentation {
        let request = PolicyRequest {
            context: context(),
            action,
            egress: Vec::new(),
            capability: None,
            destination_requested_by_user: false,
        };
        for_handoff(
            &request,
            RuleId::UnknownRequiresHandoff,
            ActionClass::Unknown,
        )
    }

    #[test]
    fn navigation_presentation_redacts_credentials_in_the_url() {
        let action = ValidatedAction::new(Action::Navigate(NavigateParams {
            url: BoundedText::parse(
                "https://example.com/checkout?token=abcd1234",
                "navigate.url",
                2_048,
            )
            .unwrap(),
        }))
        .unwrap();
        let presentation = present(&action);
        let rendered = format!("{:?}", presentation.parameters);
        assert!(!rendered.contains("abcd1234"));
        assert!(rendered.contains("token=[redacted]"));
    }

    #[test]
    fn typed_secret_shaped_text_is_masked_in_the_presentation() {
        let action = ValidatedAction::new(Action::Type(TypeParams {
            target: target(),
            text: BoundedText::parse("my password: hunter2", "type.text", 8_000).unwrap(),
            submit: false,
        }))
        .unwrap();
        let presentation = present(&action);
        assert!(
            presentation
                .parameters
                .iter()
                .any(|parameter| parameter.value.contains("[redacted]"))
        );
    }

    #[test]
    fn secure_fill_presentation_carries_only_the_opaque_reference() {
        let action = ValidatedAction::new(Action::SecureFill(SecureFillParams {
            target: target(),
            credential: CredentialRef::parse("vault-entry-7").unwrap(),
        }))
        .unwrap();
        let presentation = present(&action);
        assert_eq!(presentation.parameters[0].value, "vault-entry-7");
        assert_eq!(presentation.parameters[1].value, "https://example.com");
    }

    #[test]
    fn action_presentation_includes_undo_when_evidence_exists() {
        let action = ValidatedAction::new(Action::Type(TypeParams {
            target: target(),
            text: BoundedText::parse("running shoes", "type.text", 8_000).unwrap(),
            submit: false,
        }))
        .unwrap();
        let registry = crate::CapabilityRegistry::builtin();
        let capability = registry.select(ActionKind::Type, "textbox");
        let request = PolicyRequest {
            context: context(),
            action: &action,
            egress: Vec::new(),
            capability,
            destination_requested_by_user: false,
        };
        let presentation = for_action(
            &request,
            RuleId::ReversibleWriteNeedsApproval,
            ActionClass::ReversibleWrite,
        );
        let undo = presentation.undo_description.unwrap_or_default();
        assert!(undo.contains("field.clear"));
        assert!(undo.contains("empty as recorded before the write"));
    }

    #[test]
    fn handoff_copy_distinguishes_unknown_from_irreversible() {
        let action = ValidatedAction::new(Action::Click(ClickParams {
            target: target(),
            opens_new_tab: false,
            may_submit: false,
        }))
        .unwrap();
        let request = PolicyRequest {
            context: context(),
            action: &action,
            egress: Vec::new(),
            capability: None,
            destination_requested_by_user: false,
        };
        let unknown = for_handoff(
            &request,
            RuleId::UnknownRequiresHandoff,
            ActionClass::Unknown,
        );
        let irreversible = for_handoff(
            &request,
            RuleId::IrreversibleRequiresHandoff,
            ActionClass::Irreversible,
        );
        assert!(unknown.consequences.contains("cannot establish"));
        assert!(irreversible.consequences.contains("lasting effect"));
        assert!(
            unknown
                .target_label
                .unwrap_or_default()
                .contains("Card number")
        );
    }
}
