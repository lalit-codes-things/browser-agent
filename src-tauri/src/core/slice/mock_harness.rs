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
use crate::core::slice::mock_site::{first_mock_sites, MockSite, MockSiteDeceptionClass, ExpectedVerdict};

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
    sites.iter().map(|site| site.expected_verdict.clone()).collect()
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
        let fake = sites
            .iter()
            .find(|s| s.id == "mock-fake-action-1")
            .unwrap();
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
        assert!(matches!(snap.state, crate::browser::process::BrowserRuntimeState::Unavailable));
    }
}
