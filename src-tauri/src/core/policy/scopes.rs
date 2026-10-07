// Credential scopes.
//
// C-27: exact-origin credentials are the default; multi-subdomain scope
//        requires an explicit scope object and wildcards are restrictive.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CredentialScope {
    ExactOrigin { origin: String },
    MultiSubdomain { domain: String },
}

impl CredentialScope {
    pub fn origin_for_display(&self) -> String {
        match self {
            CredentialScope::ExactOrigin { origin } => origin.clone(),
            CredentialScope::MultiSubdomain { domain } => format!("*.{}", domain),
        }
    }
}
