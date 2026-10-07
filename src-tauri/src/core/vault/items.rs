// Vault item metadata.
//
// C-143: raw secrets, cookies, authentication headers, and secret values
//        never enter model context or verification evidence.
// C-148: searchable origin metadata uses a domain-separated blind index
//        (core/vault/blind_index) instead of storing plaintext origin in a
//        way that leaks identity if the SQLite portion is extracted without
//        the vault key.
//
// This module surfaces the vault item descriptor the UI can display; the
// raw secret itself stays in the zeroized secret store and never crosses
// into React/Tauri IPC.

use serde::{Deserialize, Serialize};

use crate::core::policy::scopes::CredentialScope;
use crate::core::verification::redaction::RedactedSecretDescriptor;

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

/// A reference to a vault item as visible to the UI layer. This does NOT
/// contain the raw secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultItemRef {
    pub id: String,
    pub origin: String,
    pub account_label: String,
    pub scope: CredentialScope,
    pub https_check_state: HttpsCheckState,
    pub idn_homograph_check_state: IdnHomographCheckState,
    pub last_used_task: Option<String>,
    /// If this item is a payment credential, the redacted secret descriptor
    /// is provided here instead of the raw secret.
    pub redacted_secret: Option<RedactedSecretDescriptor>,
}

impl VaultItemRef {
    pub fn payment_item_ref(
        id: String,
        origin: String,
        account_label: String,
        https_check_state: HttpsCheckState,
        idn_homograph_check_state: IdnHomographCheckState,
        redacted: RedactedSecretDescriptor,
    ) -> Self {
        Self {
            id,
            origin,
            account_label,
            scope: CredentialScope::PaymentToken,
            https_check_state,
            idn_homograph_check_state,
            last_used_task: None,
            redacted_secret: Some(redacted),
        }
    }
}

/// A blind-index lookup value for an origin, computed under the vault blind-
/// index key. The lookup value is what is stored in the DB for origin lookup;
/// it does not contain the plaintext origin and does not contain the blind-
/// index key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlindIndexValue {
    pub value: [u8; 32],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_item_ref_does_not_expose_raw_secret() {
        let ref_ = VaultItemRef::payment_item_ref(
            "v-1".into(),
            "https://payee.example".into(),
            "Payee".into(),
            HttpsCheckState::Pass,
            IdnHomographCheckState::Pass,
            RedactedSecretDescriptor::payment_field("card number"),
        );
        assert!(format!("{:?}", ref_).contains("Payee"));
        assert!(format!("{:?}", ref_).contains("[redacted]"));
    }
}
