// Canonical request representation.
//
// C-72: correlation key uses requestId, destination, method, body digest where
//        available.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalRequest {
    pub request_id: String,
    pub destination: String,
    pub method: String,
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub body_digest: Option<String>,
    pub classification: Option<crate::net::classify::RequestClass>,
}
