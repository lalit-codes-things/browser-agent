// Vault header + Argon2id parameters.
//
// C-148: vault key hierarchy: master password -> Argon2id KEK -> keychain-
//        resident master key wrapping distinct vault-item and audit keys;
//        password change re-wraps keys without breaking HMAC chain verification
//        across rotated audit segments.
//
// At vault creation the runtime calibrates Argon2id against the target
// environment, enforces a minimum security floor, persists the chosen
// parameters in the vault header, and never silently weakens an existing
// vault.

use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Argon2Variant {
    Argon2id,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Argon2Parameters {
    pub variant: Argon2Variant,
    pub version: u32,
    pub memory_cost: u32,
    pub time_cost: u32,
    pub parallelism: u32,
    pub calibration_version: u32,
}

impl Argon2Parameters {
    /// Minimum security floor for vault creation.
    ///
    /// These bounds are the application floor; an existing vault keeps the
    /// parameters it was created with. New vaults created after this floor
    /// was set MUST meet at least these bounds.
    pub fn minimum_floor() -> Self {
        Self {
            variant: Argon2Variant::Argon2id,
            version: 19,
            memory_cost: 64 * 1024,
            time_cost: 2,
            parallelism: 1,
            calibration_version: 1,
        }
    }

    /// A parameters object is at-or-above floor if every field meets the
    /// minimum. Missing/malformed/below-floor params are rejected at creation
    /// time.
    pub fn meets_floor(&self) -> bool {
        self.variant == Argon2Variant::Argon2id
            && self.version >= Self::minimum_floor().version
            && self.memory_cost >= Self::minimum_floor().memory_cost
            && self.time_cost >= Self::minimum_floor().time_cost
            && self.parallelism >= Self::minimum_floor().parallelism
    }

    pub fn meets_or_above(&self, other: &Self) -> bool {
        self.variant == other.variant
            && self.version >= other.version
            && self.memory_cost >= other.memory_cost
            && self.time_cost >= other.time_cost
            && self.parallelism >= other.parallelism
    }

    pub fn serialized_minimal_repr(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(b"argon2id\0");
        v.extend_from_slice(&self.version.to_be_bytes());
        v.extend_from_slice(&self.memory_cost.to_be_bytes());
        v.extend_from_slice(&self.time_cost.to_be_bytes());
        v.extend_from_slice(&self.parallelism.to_be_bytes());
        v.extend_from_slice(&self.calibration_version.to_be_bytes());
        v
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalibrationOutcome {
    Parameters(Argon2Parameters),
    BelowFloorRejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultHeader {
    pub version: u32,
    pub algorithm: String,
    pub parameters: Argon2Parameters,
    pub created_at_monotonic: u64,
    pub salt: [u8; 16],
    pub wrapped_master_key: Vec<u8>,
    pub header_tag: [u8; 32],
}

impl VaultHeader {
    pub fn new(parameters: Argon2Parameters, now_monotonic: u64) -> Result<Self, crate::Error> {
        if !parameters.meets_floor() {
            return Err(crate::Error::InvalidParameter(
                "Argon2id parameters below minimum vault floor".into(),
            ));
        }
        Ok(Self {
            version: 1,
            algorithm: "argon2id".into(),
            parameters,
            created_at_monotonic: now_monotonic,
            salt: {
                let mut salt = [0u8; 16];
                rand::thread_rng().fill_bytes(&mut salt);
                salt
            },
            wrapped_master_key: Vec::new(),
            header_tag: [0u8; 32],
        })
    }

    pub fn recalibration_performed(
        &self,
        new_parameters: &Argon2Parameters,
        _now_monotonic: u64,
    ) -> bool {
        new_parameters.meets_or_above(&self.parameters) && new_parameters.meets_floor()
    }
}

/// Attempt to calibrate Argon2id against the current environment and return
/// parameters that meet the floor, or reject if the environment cannot meet
/// the floor.
///
/// This is a simplified calibration model: the real production calibration
/// would measure wall-clock time for a representative Argon2id invocation
/// and tune time_cost/memory_cost to a target p95. Here we enforce the floor
/// and leave the exact tuner as the thing to implement when the platform
/// primitive is available.
pub fn calibrate_argon2id(
    _measured_latency_ms: Option<f64>,
    environment_memory_kb: Option<u64>,
) -> CalibrationOutcome {
    // Use the floor as the default target for a vault being created now.
    let floor = Argon2Parameters::minimum_floor();
    // If the environment is too constrained to even host the floor, reject.
    if let Some(mem) = environment_memory_kb {
        // floor.memory_cost is in KiB.
        if mem < floor.memory_cost as u64 {
            return CalibrationOutcome::BelowFloorRejected;
        }
    }
    // Otherwise return the floor as the calibrated target for this vault
    // creation. In a real calibration pass we would tune up from the floor
    // toward a latency target; that tuner is explicitly scheduled and not
    // faked here.
    CalibrationOutcome::Parameters(floor)
}

#[derive(Debug, thiserror::Error)]
pub enum VaultParameterError {
    #[error("Argon2id parameters missing or malformed")]
    Malformed,
    #[error("Argon2id parameters below minimum vault floor")]
    BelowFloor,
    #[error("downgrade of vault parameters is rejected")]
    Downgrade,
    #[error("unsupported vault parameter variant")]
    Unsupported,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimum_floor_meets_itself() {
        let floor = Argon2Parameters::minimum_floor();
        assert!(floor.meets_floor());
        assert!(floor.meets_or_above(&floor));
    }

    #[test]
    fn below_memory_floor_is_rejected() {
        let weak = Argon2Parameters {
            variant: Argon2Variant::Argon2id,
            version: 19,
            memory_cost: 1024,
            time_cost: 2,
            parallelism: 1,
            calibration_version: 1,
        };
        assert!(!weak.meets_floor());
    }

    #[test]
    fn calibrate_returns_floor_on_capable_environment() {
        let outcome = calibrate_argon2id(Some(10.0), Some(1_000_000));
        match outcome {
            CalibrationOutcome::Parameters(p) => {
                assert!(p.meets_floor());
                assert_eq!(p.variant, Argon2Variant::Argon2id);
                assert_eq!(p.memory_cost, 64 * 1024);
            }
            CalibrationOutcome::BelowFloorRejected => panic!("environment should be capable"),
        }
    }

    #[test]
    fn insufficient_memory_rejects() {
        let outcome = calibrate_argon2id(Some(10.0), Some(1024));
        assert_eq!(outcome, CalibrationOutcome::BelowFloorRejected);
    }

    #[test]
    fn vault_header_rejects_below_floor() {
        let weak = Argon2Parameters {
            variant: Argon2Variant::Argon2id,
            version: 19,
            memory_cost: 1024,
            time_cost: 2,
            parallelism: 1,
            calibration_version: 1,
        };
        // new() must reject below floor.
        assert!(VaultHeader::new(weak, 1_000_000).is_err());
        let good = Argon2Parameters::minimum_floor();
        assert!(VaultHeader::new(good, 1_000_000).is_ok());
    }

    #[test]
    fn deserialization_preserves_floor_check_state() {
        let p = Argon2Parameters::minimum_floor();
        let json = serde_json::to_string(&p).unwrap();
        let back: Argon2Parameters = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p);
        assert!(back.meets_floor());
    }
}
