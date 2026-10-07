// Privileged pf helper.
//
// C-35 (network egress / pf enforcement boundary): the helper applies
// packet-filter rules under privilege separation. The main application
// never runs as root; enforcement failure is fail-closed.
//
// Phase 2 boundary stub: the request/response protocol is typed here so
// the application side can integrate against a stable contract. Rule
// application requires the privileged install path (helper/install) and
// is intentionally not implemented inline.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PfOperation {
    /// Apply the egress rule set.
    ApplyRules,
    /// Remove all rules installed by this helper.
    ClearRules,
    /// Report current enforcement state.
    Status,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PfRequest {
    pub operation: PfOperation,
    /// Opaque rule-set blob produced by the trusted policy layer.
    /// The helper does not interpret rule semantics.
    pub rules: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PfEnforcementState {
    Enforced,
    NotEnforced,
    /// Helper could not determine state: fail-closed consumers treat
    /// this as not-enforced.
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PfResponse {
    pub state: PfEnforcementState,
    pub detail: Option<String>,
}

pub fn handle(request: PfRequest) -> PfResponse {
    // Privileged execution path is scheduled with the Phase 2 pf
    // boundary. Every response here is honest: nothing is enforced by
    // this stub, so state is Unknown, never Enforced.
    let _ = request;
    PfResponse {
        state: PfEnforcementState::Unknown,
        detail: Some("pf enforcement path not installed (Phase 2)".into()),
    }
}

fn main() {
    // Stdin/stdout framed JSON protocol for launchd-mediated invocation.
    // The full privilege-separated transport is the Phase 2 deliverable;
    // this binary is buildable and testable now without faking enforcement.
    let initialized = tracing_subscriber_fmt();
    let _ = initialized;

    let exit_code = match read_request() {
        Some(request) => {
            let response = handle(request);
            match serde_json::to_writer(std::io::stdout(), &response) {
                Ok(()) => 0,
                Err(_) => 1,
            }
        }
        None => 1,
    };
    std::process::exit(exit_code);
}

fn tracing_subscriber_fmt() -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    // Minimal subscriber; the helper must stay dependency-lean.
    tracing::subscriber::set_global_default(tracing::subscriber::NoSubscriber::new())
}

fn read_request() -> Option<PfRequest> {
    use std::io::Read;
    let mut buf = String::new();
    match std::io::stdin().read_to_string(&mut buf) {
        Ok(0) => None,
        Ok(_) => serde_json::from_str(&buf).ok(),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_never_claims_enforcement() {
        let response = handle(PfRequest { operation: PfOperation::Status, rules: None });
        assert_eq!(response.state, PfEnforcementState::Unknown);
    }

    #[test]
    fn request_roundtrips() {
        let request = PfRequest { operation: PfOperation::ApplyRules, rules: Some("block drop proto udp to any port 443".into()) };
        let json = serde_json::to_string(&request).unwrap();
        let back: PfRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, request);
    }
}
