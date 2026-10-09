// pf.
//
// C-73, C-74: privileged pf helper is signed/versioned and installed via
//        SMAppService; degradation mode honestly documented when absent.

pub struct PfHelper;

impl PfHelper {
    pub fn install_degradation_present() -> bool {
        false
    }

    pub fn enforce_egress_rule(_rule: &str) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented(
            "PfHelper::enforce_egress_rule is scheduled".into(),
        ))
    }
}
