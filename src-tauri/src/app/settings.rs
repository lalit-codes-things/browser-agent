// Settings.
//
// Local runtime settings only (no account/billing/team/cloud-sync settings).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub runtime: RuntimeSettings,
    pub browser: BrowserSettings,
    pub model: ModelSettings,
    pub network: NetworkSettings,
    pub vault: VaultSettings,
    pub security: SecuritySettings,
    pub audit: AuditSettings,
    pub keyboard: KeyboardSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSettings {
    pub announce_latency: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSettings {
    pub headless_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSettings {
    pub use_provisional_model: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub egress_mode: crate::config::EgressMode,
    pub quic_blocked: bool,
    pub doh_disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSettings {
    pub master_key_storage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub high_stakes_threshold_amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSettings {
    pub retention_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardSettings {
    pub reduced_motion: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            runtime: RuntimeSettings {
                announce_latency: true,
            },
            browser: BrowserSettings {
                headless_allowed: true,
            },
            model: ModelSettings {
                use_provisional_model: true,
            },
            network: NetworkSettings {
                egress_mode: crate::config::EgressMode::Enforced,
                quic_blocked: true,
                doh_disabled: true,
            },
            vault: VaultSettings {
                master_key_storage: "keychain".into(),
            },
            security: SecuritySettings {
                high_stakes_threshold_amount: None,
            },
            audit: AuditSettings {
                retention_policy: "strict_tamper_evident".into(),
            },
            keyboard: KeyboardSettings {
                reduced_motion: false,
            },
        }
    }
}
