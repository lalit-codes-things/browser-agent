// Vault item metadata.
//
// C-143: raw secrets, cookies, authentication headers, and secret values
//        never enter model context or verification evidence.
// C-27, C-28, C-29: credential scoping, (origin, account) pairing, and
//        disambiguation rules.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultItemRef {
    pub origin: String,
    pub account_label: String,
    pub scope: crate::core::policy::scopes::CredentialScope,
    pub https_check_state: HttpsCheckState,
    pub idn_homograph_check_state: IdnHomographCheckState,
    pub last_used_task: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpsCheckState {
    Pass,
    Fail,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum IdnHomographCheckState {
    Pass,
    Mismatch,
    Unknown,
}
