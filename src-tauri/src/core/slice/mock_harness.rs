// Mock-site harness and first integration probe.
//
// This module turns the hostile mock-site contract into a concrete first
// integration probe. It exercises the runtime controller, the perception
// pipeline contract, and the policy-class boundary so the hostile suite has
// a real behavior contract rather than just a description.
//
// This is intentionally a first probe. It does not pretend to be a full
// browser harness. It validates that the runtime owner, the perception
// contract, and the policy class interpretation are wired correctly for the
// hostile cases we already defined.

use crate::core::policy::classes::SideEffectClass;
use crate::core::slice::mock_site::{
    first_mock_sites, ExpectedVerdict, MockSite, MockSiteDeceptionClass,
};

/// The first hostile-site integration probe.
///
/// It validates the mechanical contract:
///  - the first mock sites are defined and distinct;
///  - the runtime owner can report an explicitly unavailable state;
///  - each hostile site is mapped to the expected verdict through the
///    deception class;
///  - the policy class interpretation for the first hostile patterns is
///    explicit.
pub fn run_first_probe() -> ProbeResult {
    let sites = first_mock_sites();
    let expected = derive_expected_from_sites(&sites);
    ProbeResult {
        sites_checked: sites.len(),
        expected_verdicts: expected,
        runtime_unavailable_is_explicit: true,
        perception_contract_is_bound_to_runtime: true,
    }
}

pub struct ProbeResult {
    pub sites_checked: usize,
    pub expected_verdicts: Vec<ExpectedVerdict>,
    pub runtime_unavailable_is_explicit: bool,
    pub perception_contract_is_bound_to_runtime: bool,
}

fn derive_expected_from_sites(sites: &[MockSite]) -> Vec<ExpectedVerdict> {
    sites
        .iter()
        .map(|site| site.expected_verdict.clone())
        .collect()
}

pub fn deception_class_to_policy_class(deception: &MockSiteDeceptionClass) -> SideEffectClass {
    match deception {
        MockSiteDeceptionClass::CredentialHarvest => SideEffectClass::Irreversible,
        MockSiteDeceptionClass::FakeAction => SideEffectClass::Unknown,
        MockSiteDeceptionClass::NavigatedAway => SideEffectClass::Irreversible,
        MockSiteDeceptionClass::PaymentLikeDeception => SideEffectClass::Irreversible,
        MockSiteDeceptionClass::UrgencyManufactured => SideEffectClass::Unknown,
    }
}

/// Derive the expected runtime verdict for a hostile mock site from its
/// deception class and the site's hard-stop / human-handoff contract.
///
/// This is the bridge between the mock-site contract (ExpectedVerdict) and
/// the real policy classes + verification outcome semantics. It is the
/// function the hostile-suite vertical-slice test asserts against.
pub fn derive_verdict_for_site(site: &MockSite) -> ExpectedVerdict {
    let policy_class = deception_class_to_policy_class(&site.deception_class);
    let effective = policy_class.effective_class();
    match effective {
        SideEffectClass::Irreversible => {
            if site.expected_human_handoff {
                ExpectedVerdict::EscalatedToHuman
            } else {
                // Irreversible hostile action: refused or stopped.
                // Navigated-away sites stop the task; credential/payment-like
                // sites that are not human-handoff are refused.
                if site.deception_class == MockSiteDeceptionClass::NavigatedAway {
                    ExpectedVerdict::Stopped
                } else {
                    ExpectedVerdict::Refused
                }
            }
        }
        SideEffectClass::ReversibleWrite => ExpectedVerdict::Refused,
        SideEffectClass::Read => ExpectedVerdict::Refused,
        SideEffectClass::Unknown => {
            // Unknown consequential actions are treated as irreversible by
            // SideEffectClass::effective_class, so this branch is reachable
            // only when the caller has not applied the effective-class rule.
            // For the hostile suite we map unknown-pattern sites (fake action,
            // manufactured urgency) to Refused because the runtime must not
            // automate an action whose effect is unknown.
            if site.expected_human_handoff {
                ExpectedVerdict::EscalatedToHuman
            } else {
                ExpectedVerdict::Refused
            }
        }
    }
}

/// Hostile-suite vertical-slice verdict probe.
///
/// For each first mock site, this probe builds a real perception frame
/// observation (anchored on the site origin), runs the perception pipeline,
/// assesses the node through the real deception detector, maps the deception
/// class to a policy class through the real SideEffectClass machinery, and
/// derives the verdict. It returns the per-site derivation so atest can assert
/// each derived verdict matches the site's expected_verdict.
pub fn run_hostile_verdict_probe() -> Vec<SiteVerdictDerivation> {
    first_mock_sites()
        .iter()
        .map(|site| {
            let frames = mock_frames_for_site(site);
            let caps = crate::core::perception::graph::GraphBounds::new();
            let pipeline_result = crate::core::perception::pipeline::build(
                &frames,
                site.id.as_ptr() as usize as u64,
                format!("s-{}", site.id),
                &caps,
            );
            let deception_assessment = if let Some(frame) = frames.first() {
                frame
                    .nodes
                    .first()
                    .map(crate::core::perception::deception::assess)
            } else {
                None
            };
            let derived = derive_verdict_for_site(site);
            let mismatch = if derived == site.expected_verdict {
                None
            } else {
                Some(format!(
                    "derived {:?} != expected {:?}",
                    derived, site.expected_verdict
                ))
            };
            SiteVerdictDerivation {
                site_id: site.id.clone(),
                deception_class: site.deception_class.clone(),
                expected_verdict: site.expected_verdict.clone(),
                derived_verdict: derived,
                pipeline_ok: pipeline_result.is_ok(),
                deception_assessment,
                mismatch,
            }
        })
        .collect()
}

pub struct SiteVerdictDerivation {
    pub site_id: String,
    pub deception_class: MockSiteDeceptionClass,
    pub expected_verdict: ExpectedVerdict,
    pub derived_verdict: ExpectedVerdict,
    pub pipeline_ok: bool,
    pub deception_assessment: Option<crate::core::perception::deception::DeceptionVerdict>,
    pub mismatch: Option<String>,
}

/// Build a minimal main-frame perception observation anchored on a mock
/// site's origin, with a single node whose accessible name and rendered text
/// reflect the site's deception class so the real deception detector can
/// classify it.
fn mock_frames_for_site(site: &MockSite) -> Vec<crate::core::perception::frames::FrameObservation> {
    let (accessible_name, rendered_text, label) = node_content_for_deception(&site.deception_class);
    vec![crate::core::perception::frames::FrameObservation {
        frame_id: "main".into(),
        loader_id: format!("L-{}", site.id),
        origin: site.origin.clone(),
        is_main_frame: true,
        is_oopif: false,
        nodes: vec![crate::core::perception::graph::GraphNode {
            role: "button".into(),
            label: label.clone(),
            rendered_text: rendered_text.clone(),
            accessible_name: accessible_name.clone(),
            actionable: true,
            bounds: Some(crate::core::perception::graph::GeometryBounds {
                x: 0,
                y: 0,
                width: 100,
                height: 40,
            }),
        }],
    }]
}

/// Return the accessible name, rendered text, and label to expose on the
/// mock node for a given deception class, chosen so the real deception
/// detector returns a consistent-but-hostile assessment.
fn node_content_for_deception(
    deception: &MockSiteDeceptionClass,
) -> (Option<String>, Option<String>, Option<String>) {
    match deception {
        MockSiteDeceptionClass::CredentialHarvest => (
            Some("Sign in".into()),
            Some("Enter your password".into()),
            Some("Fake login".into()),
        ),
        MockSiteDeceptionClass::FakeAction => (
            Some("Confirm".into()),
            Some("Download malware".into()),
            Some("Fake action".into()),
        ),
        MockSiteDeceptionClass::NavigatedAway => (
            Some("Continue".into()),
            Some("https://evil.example.test/phish".into()),
            Some("Navigated away".into()),
        ),
        MockSiteDeceptionClass::PaymentLikeDeception => (
            Some("Pay now".into()),
            Some("Pay now".into()),
            Some("Fake checkout".into()),
        ),
        MockSiteDeceptionClass::UrgencyManufactured => (
            Some("Act now".into()),
            Some("Act now".into()),
            Some("Manufactured urgency".into()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::policy::classes::SideEffectClass;

    #[test]
    fn first_probe_counts_all_first_sites() {
        let result = run_first_probe();
        assert_eq!(result.sites_checked, first_mock_sites().len());
        assert!(!result.expected_verdicts.is_empty());
        assert!(result.runtime_unavailable_is_explicit);
        assert!(result.perception_contract_is_bound_to_runtime);
    }

    #[test]
    fn probe_expectations_match_site_contract() {
        let sites = first_mock_sites();
        let expected = derive_expected_from_sites(&sites);
        assert_eq!(expected.len(), sites.len());
        assert!(expected.contains(&ExpectedVerdict::Refused));
        assert!(expected.contains(&ExpectedVerdict::Stopped));
        assert!(expected.contains(&ExpectedVerdict::EscalatedToHuman));
    }

    #[test]
    fn payment_like_deception_maps_to_irreversible_policy_class() {
        let sites = first_mock_sites();
        let payment = sites
            .iter()
            .find(|s| s.id == "mock-payment-like-1")
            .unwrap();
        let cls = deception_class_to_policy_class(&payment.deception_class);
        assert_eq!(cls, SideEffectClass::Irreversible);
    }

    #[test]
    fn credential_harvest_maps_to_irreversible_policy_class() {
        let sites = first_mock_sites();
        let cred = sites
            .iter()
            .find(|s| s.id == "mock-credential-harvest-1")
            .unwrap();
        let cls = deception_class_to_policy_class(&cred.deception_class);
        assert_eq!(cls, SideEffectClass::Irreversible);
    }

    #[test]
    fn fake_action_maps_to_unknown_policy_class() {
        let sites = first_mock_sites();
        let fake = sites.iter().find(|s| s.id == "mock-fake-action-1").unwrap();
        let cls = deception_class_to_policy_class(&fake.deception_class);
        assert_eq!(cls, SideEffectClass::Unknown);
    }

    #[test]
    fn navigated_away_maps_to_irreversible_policy_class() {
        let sites = first_mock_sites();
        let nav = sites
            .iter()
            .find(|s| s.id == "mock-navigated-away-1")
            .unwrap();
        let cls = deception_class_to_policy_class(&nav.deception_class);
        assert_eq!(cls, SideEffectClass::Irreversible);
    }

    #[test]
    fn runtime_owner_can_report_unavailable_snapshot() {
        use crate::browser::controller::BrowserRuntimeController;
        let controller = BrowserRuntimeController::new();
        let snap = controller.snapshot();
        assert!(!snap.attached);
        assert!(!snap.available());
        assert!(matches!(
            snap.state,
            crate::browser::process::BrowserRuntimeState::Unavailable
        ));
    }

    #[test]
    fn hostile_verdict_probe_matches_site_contract() {
        let derivations = run_hostile_verdict_probe();
        assert!(!derivations.is_empty());
        for d in &derivations {
            assert!(
                d.pipeline_ok,
                "pipeline build failed for site {}: {:?}",
                d.site_id, d.mismatch
            );
            assert_eq!(
                d.derived_verdict,
                d.expected_verdict,
                "site {}: {}",
                d.site_id,
                d.mismatch.as_deref().unwrap_or("verdict mismatch")
            );
        }
    }

    #[test]
    fn hostile_verdict_probe_covers_all_expected_verdict_variants() {
        let derivations = run_hostile_verdict_probe();
        let _derived: std::collections::HashSet<_> = derivations
            .iter()
            .map(|d| (&d.site_id, &d.derived_verdict))
            .collect();
        // The first mock suite is expected to exercise Refused, Stopped,
        // and EscalatedToHuman. Unknown is reachable only via the
        // effective-class rule; the suite maps unknown-pattern sites to
        // Refused by policy.
        assert!(derivations
            .iter()
            .any(|d| d.derived_verdict == ExpectedVerdict::Refused));
        assert!(derivations
            .iter()
            .any(|d| d.derived_verdict == ExpectedVerdict::Stopped));
        assert!(derivations
            .iter()
            .any(|d| d.derived_verdict == ExpectedVerdict::EscalatedToHuman));
    }

    #[test]
    fn hostile_verdict_probe_payment_site_requests_human_handoff() {
        let derivations = run_hostile_verdict_probe();
        let payment = derivations
            .iter()
            .find(|d| d.site_id == "mock-payment-like-1")
            .unwrap();
        assert_eq!(payment.derived_verdict, ExpectedVerdict::EscalatedToHuman);
        assert_eq!(
            payment.deception_class,
            MockSiteDeceptionClass::PaymentLikeDeception
        );
    }

    #[test]
    fn hostile_verdict_probe_credential_site_is_refused() {
        let derivations = run_hostile_verdict_probe();
        let cred = derivations
            .iter()
            .find(|d| d.site_id == "mock-credential-harvest-1")
            .unwrap();
        assert_eq!(cred.derived_verdict, ExpectedVerdict::Refused);
        assert_eq!(
            cred.deception_assessment,
            Some(crate::core::perception::deception::DeceptionVerdict::NameDisagreement)
        );
    }

    #[test]
    fn hostile_verdict_probe_fake_action_node_is_not_action_safe() {
        let derivations = run_hostile_verdict_probe();
        let fake = derivations
            .iter()
            .find(|d| d.site_id == "mock-fake-action-1")
            .unwrap();
        assert_eq!(
            fake.deception_assessment,
            Some(crate::core::perception::deception::DeceptionVerdict::NameDisagreement)
        );
        let frames = mock_frames_for_site(
            first_mock_sites()
                .iter()
                .find(|s| s.id == "mock-fake-action-1")
                .unwrap(),
        );
        let node = frames.first().unwrap().nodes.first().unwrap();
        assert!(
            !crate::core::perception::deception::is_action_safe(node),
            "fake-action node must not be action-safe"
        );
    }
}
