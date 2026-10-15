//! Trusted action classification (Section 13.2).
//!
//! Classification is a runtime policy result, never a label chosen by the
//! model. Hints that arrive with the action (for example `may_submit`) can only
//! make the outcome stricter: a hint may turn `REVERSIBLE_WRITE` into
//! `IRREVERSIBLE`, never the reverse.

use ba_domain::{
    Action, ActionClass, ActionKind, UnixMillis, is_secret_like, url_discloses_credentials,
};

use crate::capability::CapabilityRecord;

/// Everything classification is allowed to look at.
#[derive(Debug, Clone, Copy)]
pub struct ClassifyInput<'a> {
    /// Action under evaluation.
    pub action: &'a Action,
    /// Capability evidence, when a record covers this operation.
    pub capability: Option<&'a CapabilityRecord>,
    /// Current time, used for capability expiry.
    pub now: UnixMillis,
    /// Whether the user enabled credential use for the exact origin.
    pub secure_fill_authorized: bool,
}

/// Returns the trusted consequence class of an action.
#[must_use]
pub fn classify(input: &ClassifyInput<'_>) -> ActionClass {
    let ClassifyInput {
        action,
        capability,
        now,
        ..
    } = *input;

    match action {
        Action::Wait(_) | Action::Scroll(_) => ActionClass::Read,
        Action::RequestConfirmation(_) | Action::HandoffToUser(_) => {
            // Control-plane actions with no page effect of their own.
            ActionClass::Read
        }
        Action::Navigate(params) => {
            let url = params.url.as_str();
            if url_discloses_credentials(url) || is_secret_like(url) {
                return ActionClass::Irreversible;
            }
            ActionClass::Read
        }
        Action::Type(params) => {
            // Submitting a form or typing secret-looking text is consequential.
            if params.submit || is_secret_like(params.text.as_str()) {
                return ActionClass::Irreversible;
            }
            match capability {
                Some(record) if record.check_valid_at(now).is_ok() => ActionClass::ReversibleWrite,
                _ => ActionClass::Unknown,
            }
        }
        Action::Select(_) => match capability {
            Some(record) if record.check_valid_at(now).is_ok() => ActionClass::ReversibleWrite,
            _ => ActionClass::Unknown,
        },
        Action::Click(params) => {
            // A click that may submit or open a new target is consequential
            // regardless of any capability record.
            if params.may_submit || params.opens_new_tab {
                return ActionClass::Irreversible;
            }
            match capability {
                Some(record)
                    if record.matches_action(ActionKind::Click)
                        && record.check_valid_at(now).is_ok() =>
                {
                    ActionClass::ReversibleWrite
                }
                _ => ActionClass::Unknown,
            }
        }
        Action::SecureFill(_) => ActionClass::Irreversible,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ba_domain::{
        BoundedText, ClickParams, NavigateParams, ScrollDirection, ScrollParams, SecureFillParams,
        SelectParams, TargetRef, TypeParams, WaitParams,
    };
    use ba_domain::{CredentialRef, FrameId, NodeId, ObservationId, Origin, TabId};

    fn now() -> UnixMillis {
        UnixMillis::from_millis(1_700_000_000_000)
    }

    fn target() -> TargetRef {
        TargetRef {
            node: NodeId::parse("node-1").unwrap(),
            frame: FrameId::parse("frame-0").unwrap(),
            tab: TabId::parse("tab-0").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            role: Some(BoundedText::parse("textbox", "target.role", 128).unwrap()),
            label: Some(BoundedText::parse("Search", "target.label", 512).unwrap()),
        }
    }

    fn text(value: &str) -> BoundedText {
        BoundedText::parse(value, "text", 8_000).unwrap()
    }

    #[test]
    fn read_actions_stay_read() {
        let registry = crate::CapabilityRegistry::builtin();
        for action in [
            Action::Wait(WaitParams { max_ms: 500 }),
            Action::Scroll(ScrollParams {
                direction: ScrollDirection::Down,
                amount: 1,
            }),
        ] {
            let class = classify(&ClassifyInput {
                action: &action,
                capability: registry.select(action.kind(), "textbox"),
                now: now(),
                secure_fill_authorized: false,
            });
            assert_eq!(class, ActionClass::Read, "{}", action.kind());
        }
    }

    #[test]
    fn plain_navigation_is_read_but_credential_urls_are_not() {
        let plain = Action::Navigate(NavigateParams {
            url: text("https://example.com/search?q=running+shoes"),
        });
        assert_eq!(
            classify(&ClassifyInput {
                action: &plain,
                capability: None,
                now: now(),
                secure_fill_authorized: false,
            }),
            ActionClass::Read
        );

        let leaky = Action::Navigate(NavigateParams {
            url: text("https://example.com/?token=abc123"),
        });
        assert_eq!(
            classify(&ClassifyInput {
                action: &leaky,
                capability: None,
                now: now(),
                secure_fill_authorized: false,
            }),
            ActionClass::Irreversible
        );
    }

    #[test]
    fn typing_is_reversible_only_with_a_capability_record() {
        let action = Action::Type(TypeParams {
            target: target(),
            text: text("running shoes"),
            submit: false,
        });
        let registry = crate::CapabilityRegistry::builtin();
        assert_eq!(
            classify(&ClassifyInput {
                action: &action,
                capability: registry.select(ActionKind::Type, "textbox"),
                now: now(),
                secure_fill_authorized: false,
            }),
            ActionClass::ReversibleWrite
        );
        assert_eq!(
            classify(&ClassifyInput {
                action: &action,
                capability: None,
                now: now(),
                secure_fill_authorized: false,
            }),
            ActionClass::Unknown
        );
    }

    #[test]
    fn submitting_or_typing_a_secret_is_irreversible() {
        let registry = crate::CapabilityRegistry::builtin();
        let capability = registry.select(ActionKind::Type, "textbox");
        let submitting = Action::Type(TypeParams {
            target: target(),
            text: text("running shoes"),
            submit: true,
        });
        assert_eq!(
            classify(&ClassifyInput {
                action: &submitting,
                capability,
                now: now(),
                secure_fill_authorized: false,
            }),
            ActionClass::Irreversible
        );

        let secret = Action::Type(TypeParams {
            target: target(),
            text: text("my password: hunter2"),
            submit: false,
        });
        assert_eq!(
            classify(&ClassifyInput {
                action: &secret,
                capability,
                now: now(),
                secure_fill_authorized: false,
            }),
            ActionClass::Irreversible
        );
    }

    #[test]
    fn click_hints_can_only_tighten() {
        let registry = crate::CapabilityRegistry::builtin();
        let plain = Action::Click(ClickParams {
            target: target(),
            opens_new_tab: false,
            may_submit: false,
        });
        assert_eq!(
            classify(&ClassifyInput {
                action: &plain,
                capability: registry.select(ActionKind::Click, "textbox"),
                now: now(),
                secure_fill_authorized: false,
            }),
            ActionClass::Unknown,
            "no click capability exists, so the class must fail closed"
        );

        let submitting = Action::Click(ClickParams {
            target: target(),
            opens_new_tab: false,
            may_submit: true,
        });
        assert_eq!(
            classify(&ClassifyInput {
                action: &submitting,
                capability: registry.select(ActionKind::Click, "textbox"),
                now: now(),
                secure_fill_authorized: false,
            }),
            ActionClass::Irreversible
        );
    }

    #[test]
    fn select_and_secure_fill_classification() {
        let registry = crate::CapabilityRegistry::builtin();
        let select = Action::Select(SelectParams {
            target: target(),
            option: text("Large"),
        });
        assert_eq!(
            classify(&ClassifyInput {
                action: &select,
                capability: registry.select(ActionKind::Select, "combobox"),
                now: now(),
                secure_fill_authorized: false,
            }),
            ActionClass::ReversibleWrite
        );

        let fill = Action::SecureFill(SecureFillParams {
            target: target(),
            credential: CredentialRef::parse("vault-entry-1").unwrap(),
        });
        assert_eq!(
            classify(&ClassifyInput {
                action: &fill,
                capability: None,
                now: now(),
                secure_fill_authorized: true,
            }),
            ActionClass::Irreversible
        );
    }

    #[test]
    fn expired_capability_downgrades_to_unknown() {
        let registry = crate::CapabilityRegistry::builtin();
        let mut expired = registry.records()[0].clone();
        expired.expires_at_unix_ms = UnixMillis::from_millis(now().get() - 1);
        let action = Action::Type(TypeParams {
            target: target(),
            text: text("running shoes"),
            submit: false,
        });
        assert_eq!(
            classify(&ClassifyInput {
                action: &action,
                capability: Some(&expired),
                now: now(),
                secure_fill_authorized: false,
            }),
            ActionClass::Unknown
        );
    }
}
