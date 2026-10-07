// Intercept.
//
// C-71: mutating requests pause at CDP until proxy release; reads passively
//        observed.
// C-72: correlation miss defaults to deny.

use crate::net::canonical::CanonicalRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterceptDecision {
    Allow,
    Deny,
    PauseAtCdp,
}

pub struct InterceptEngine;

impl InterceptEngine {
    pub fn decide(_request: &CanonicalRequest, _correlation_present: bool) -> InterceptDecision {
        if !_correlation_present {
            return InterceptDecision::Deny; // C-72: correlation miss defaults to deny
        }
        match _request.method.as_str() {
            "GET" | "HEAD" => InterceptDecision::Allow,
            _ => InterceptDecision::PauseAtCdp,
        }
    }
}
