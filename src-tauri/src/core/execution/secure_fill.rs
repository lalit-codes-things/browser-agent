// Secure fill.
//
// C-13, C-143: secure_fill is a trusted runtime primitive bound to exact
//        scheme/host/port, frame, loader, field, task, and credential
//        identity; HTTPS required, certificate errors denied, punycode/
//        homograph mismatch checked; credential identity is never exposed
//        as an arbitrary list.
//
// Raw secrets never enter model context or verification evidence.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureFillBinding {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub frame_id: String,
    pub loader_id: String,
    pub field_reference: String,
    pub task_id: String,
    pub credential_identity: String,
    pub https_required: bool,
    pub certificate_check_passed: bool,
    pub idn_homograph_check_passed: bool,
}

pub struct SecureFillEngine;

impl SecureFillEngine {
    pub fn fill(_binding: SecureFillBinding) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented("SecureFillEngine::fill is scheduled".into()))
    }
}
