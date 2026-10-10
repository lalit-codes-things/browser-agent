// SQLite runtime store.
//
// Persistent local store for runtime state. Schema is release-gated (C-127).

use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;

use crate::app::settings::Settings;
use crate::audit::chain::AuditSegment;
use crate::browser::downloads::{DownloadVerificationState, QuarantinedDownload};
use crate::browser::profiles::{BrowserProfile, ProfileKind};
use crate::core::orchestrator::progress::{ProgressMode, TaskProgress, TaskState};
use crate::core::skills::store::{SkillRecord, SkillStatus};
use crate::core::vault::items::{
    BlindIndexValue, HttpsCheckState, IdnHomographCheckState, VaultItemRef,
};
use crate::error::Error;
use crate::storage::migrations::run_migrations;
use crate::storage::models::task::StoredTask;

#[derive(Clone)]
pub struct SqliteStore {
    conn: Arc<Mutex<Connection>>,
}

/// Encrypted vault secret material as stored: (encrypted_secret, nonce).
pub type EncryptedVaultSecret = (Vec<u8>, Vec<u8>);

impl SqliteStore {
    /// Open SQLite database at file path and apply migrations.
    pub fn open(path: &str) -> Result<Self, Error> {
        let path_obj = Path::new(path);
        if let Some(parent) = path_obj.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| Error::Internal(format!("failed to create db directory: {e}")))?;
            }
        }
        let conn = Connection::open(path).map_err(|e| {
            Error::Internal(format!("failed to open sqlite database at {path}: {e}"))
        })?;
        run_migrations(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Open in-memory SQLite database for testing or ephemeral profiles.
    pub fn open_in_memory() -> Result<Self, Error> {
        let conn = Connection::open_in_memory()
            .map_err(|e| Error::Internal(format!("failed to open in-memory sqlite db: {e}")))?;
        run_migrations(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    // --- Task Persistence ---

    pub fn save_task(&self, task: &StoredTask) -> Result<(), Error> {
        let conn = self.conn.lock();
        let status_str = format!("{:?}", task.status);
        let mode_str = format!("{:?}", task.progress.mode);
        let now = crate::security::clocks::MonotonicClock::now_nanos() as i64;

        conn.execute(
            "INSERT INTO tasks (id, status, step_index, total_steps, epoch, mode, summary, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
             ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                step_index = excluded.step_index,
                total_steps = excluded.total_steps,
                epoch = excluded.epoch,
                mode = excluded.mode,
                summary = excluded.summary,
                updated_at = excluded.updated_at",
            params![
                task.id,
                status_str,
                task.progress.step_index,
                task.progress.total_steps,
                task.progress.epoch as i64,
                mode_str,
                task.preserved_state_summary,
                now,
            ],
        )
        .map_err(|e| Error::Internal(format!("failed to save task: {e}")))?;
        Ok(())
    }

    pub fn get_task(&self, id: &str) -> Result<Option<StoredTask>, Error> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, status, step_index, total_steps, epoch, mode, summary FROM tasks WHERE id = ?1",
            )
            .map_err(|e| Error::Internal(format!("failed to prepare task query: {e}")))?;

        let mut rows = stmt
            .query(params![id])
            .map_err(|e| Error::Internal(format!("failed to query task: {e}")))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| Error::Internal(format!("failed to read task row: {e}")))?
        {
            let task_id: String = row.get(0).unwrap_or_default();
            let status_raw: String = row.get(1).unwrap_or_default();
            let step_index: u32 = row.get(2).unwrap_or(1);
            let total_steps: u32 = row.get(3).unwrap_or(11);
            let epoch: i64 = row.get(4).unwrap_or(1);
            let mode_raw: String = row.get(5).unwrap_or_default();
            let summary: Option<String> = row.get(6).ok();

            let status = match status_raw.as_str() {
                "Running" => TaskState::Running,
                "Verified" => TaskState::Verified,
                "LikelySuccess" => TaskState::LikelySuccess,
                "Aborted" => TaskState::Aborted,
                "Failed" => TaskState::Failed,
                _ => TaskState::Unknown,
            };

            let mode = match mode_raw.as_str() {
                "DeterministicSkill" => ProgressMode::DeterministicSkill,
                "Intervention" => ProgressMode::Intervention,
                _ => ProgressMode::ModelReasoning,
            };

            let progress = TaskProgress {
                task_id: task_id.clone(),
                current_state: status.clone(),
                step_index,
                total_steps,
                epoch: epoch as u64,
                mode,
            };

            Ok(Some(StoredTask {
                id: task_id,
                status,
                progress,
                preserved_state_summary: summary,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_tasks(&self, limit: usize) -> Result<Vec<StoredTask>, Error> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, status, step_index, total_steps, epoch, mode, summary FROM tasks ORDER BY updated_at DESC LIMIT ?1",
            )
            .map_err(|e| Error::Internal(format!("failed to prepare list tasks query: {e}")))?;

        let mut rows = stmt
            .query(params![limit as i64])
            .map_err(|e| Error::Internal(format!("failed to query tasks: {e}")))?;

        let mut out = Vec::with_capacity(limit);
        while let Some(row) = rows
            .next()
            .map_err(|e| Error::Internal(format!("failed to read task row: {e}")))?
        {
            let task_id: String = row.get(0).unwrap_or_default();
            let status_raw: String = row.get(1).unwrap_or_default();
            let step_index: u32 = row.get(2).unwrap_or(1);
            let total_steps: u32 = row.get(3).unwrap_or(11);
            let epoch: i64 = row.get(4).unwrap_or(1);
            let mode_raw: String = row.get(5).unwrap_or_default();
            let summary: Option<String> = row.get(6).ok();

            let status = match status_raw.as_str() {
                "Running" => TaskState::Running,
                "Verified" => TaskState::Verified,
                "LikelySuccess" => TaskState::LikelySuccess,
                "Aborted" => TaskState::Aborted,
                "Failed" => TaskState::Failed,
                _ => TaskState::Unknown,
            };

            let mode = match mode_raw.as_str() {
                "DeterministicSkill" => ProgressMode::DeterministicSkill,
                "Intervention" => ProgressMode::Intervention,
                _ => ProgressMode::ModelReasoning,
            };

            let progress = TaskProgress {
                task_id: task_id.clone(),
                current_state: status.clone(),
                step_index,
                total_steps,
                epoch: epoch as u64,
                mode,
            };

            out.push(StoredTask {
                id: task_id,
                status,
                progress,
                preserved_state_summary: summary,
            });
        }

        Ok(out)
    }

    // --- Audit Persistence ---

    pub fn append_audit_segment(
        &self,
        segment: &AuditSegment,
        event_type: &str,
        payload: &str,
    ) -> Result<u64, Error> {
        let conn = self.conn.lock();
        let now = crate::security::clocks::MonotonicClock::now_nanos().to_string();
        conn.execute(
            "INSERT INTO audit_records (sequence, segment_id, event_type, payload, hmac_link, segment_hmac, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                segment.sequence as i64,
                segment.segment_id,
                event_type,
                payload,
                segment.hmac_link,
                segment.segment_hmac,
                now,
            ],
        )
        .map_err(|e| Error::Internal(format!("failed to append audit record: {e}")))?;
        Ok(segment.sequence)
    }

    pub fn list_audit_segments(&self) -> Result<Vec<AuditSegment>, Error> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT segment_id, sequence, hmac_link, segment_hmac FROM audit_records ORDER BY sequence ASC",
            )
            .map_err(|e| Error::Internal(format!("failed to prepare audit segments query: {e}")))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| Error::Internal(format!("failed to query audit segments: {e}")))?;

        let mut out = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| Error::Internal(format!("failed to read audit segment row: {e}")))?
        {
            let segment_id: String = row.get(0).unwrap_or_default();
            let sequence: i64 = row.get(1).unwrap_or(0);
            let hmac_link: String = row.get(2).unwrap_or_default();
            let segment_hmac: String = row.get(3).unwrap_or_default();

            out.push(AuditSegment {
                segment_id,
                sequence: sequence as u64,
                hmac_link,
                segment_hmac,
            });
        }
        Ok(out)
    }

    // --- Vault Items ---

    pub fn save_vault_item(
        &self,
        item: &VaultItemRef,
        blind_index: &BlindIndexValue,
        encrypted_secret: &[u8],
        nonce: &[u8],
    ) -> Result<(), Error> {
        let conn = self.conn.lock();
        let scope_str = serde_json::to_string(&item.scope)
            .map_err(|e| Error::Internal(format!("failed to serialize scope: {e}")))?;
        let https_str = match item.https_check_state {
            HttpsCheckState::Pass => "PASS",
            HttpsCheckState::Fail => "FAIL",
            HttpsCheckState::Unknown => "UNKNOWN",
        };
        let idn_str = match item.idn_homograph_check_state {
            IdnHomographCheckState::Pass => "PASS",
            IdnHomographCheckState::Mismatch => "MISMATCH",
            IdnHomographCheckState::Unknown => "UNKNOWN",
        };
        let now = crate::security::clocks::MonotonicClock::now_nanos() as i64;

        conn.execute(
            "INSERT INTO vault_items (id, blind_index, origin, account_label, scope, https_check_state, idn_check_state, encrypted_secret, nonce, last_used_task, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
                origin = excluded.origin,
                account_label = excluded.account_label,
                scope = excluded.scope,
                https_check_state = excluded.https_check_state,
                idn_check_state = excluded.idn_check_state,
                encrypted_secret = excluded.encrypted_secret,
                nonce = excluded.nonce,
                last_used_task = excluded.last_used_task",
            params![
                item.id,
                blind_index.value.as_slice(),
                item.origin,
                item.account_label,
                scope_str,
                https_str,
                idn_str,
                encrypted_secret,
                nonce,
                item.last_used_task,
                now,
            ],
        )
        .map_err(|e| Error::Internal(format!("failed to save vault item: {e}")))?;
        Ok(())
    }

    pub fn list_vault_items(&self) -> Result<Vec<VaultItemRef>, Error> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, origin, account_label, scope, https_check_state, idn_check_state, last_used_task FROM vault_items ORDER BY created_at DESC",
            )
            .map_err(|e| Error::Internal(format!("failed to prepare list vault items query: {e}")))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| Error::Internal(format!("failed to query vault items: {e}")))?;

        let mut out = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| Error::Internal(format!("failed to read vault item row: {e}")))?
        {
            let id: String = row.get(0).unwrap_or_default();
            let origin: String = row.get(1).unwrap_or_default();
            let account_label: String = row.get(2).unwrap_or_default();
            let scope_raw: String = row.get(3).unwrap_or_default();
            let https_raw: String = row.get(4).unwrap_or_default();
            let idn_raw: String = row.get(5).unwrap_or_default();
            let last_used_task: Option<String> = row.get(6).ok();

            let scope = serde_json::from_str(&scope_raw).unwrap_or_else(|_| {
                crate::core::policy::scopes::CredentialScope::ExactOrigin {
                    origin: origin.clone(),
                }
            });

            let https_check_state = match https_raw.as_str() {
                "PASS" => HttpsCheckState::Pass,
                "FAIL" => HttpsCheckState::Fail,
                _ => HttpsCheckState::Unknown,
            };
            let idn_homograph_check_state = match idn_raw.as_str() {
                "PASS" => IdnHomographCheckState::Pass,
                "MISMATCH" => IdnHomographCheckState::Mismatch,
                _ => IdnHomographCheckState::Unknown,
            };

            out.push(VaultItemRef {
                id,
                origin,
                account_label,
                scope,
                https_check_state,
                idn_homograph_check_state,
                last_used_task,
                redacted_secret: None,
            });
        }
        Ok(out)
    }

    pub fn get_vault_item_secret(&self, id: &str) -> Result<Option<EncryptedVaultSecret>, Error> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare("SELECT encrypted_secret, nonce FROM vault_items WHERE id = ?1")
            .map_err(|e| Error::Internal(format!("failed to prepare secret query: {e}")))?;

        let mut rows = stmt
            .query(params![id])
            .map_err(|e| Error::Internal(format!("failed to query secret: {e}")))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| Error::Internal(format!("failed to read secret row: {e}")))?
        {
            let encrypted: Vec<u8> = row.get(0).unwrap_or_default();
            let nonce: Vec<u8> = row.get(1).unwrap_or_default();
            Ok(Some((encrypted, nonce)))
        } else {
            Ok(None)
        }
    }

    pub fn delete_vault_item(&self, id: &str) -> Result<bool, Error> {
        let conn = self.conn.lock();
        let rows = conn
            .execute("DELETE FROM vault_items WHERE id = ?1", params![id])
            .map_err(|e| Error::Internal(format!("failed to delete vault item: {e}")))?;
        Ok(rows > 0)
    }

    // --- Skills ---

    pub fn save_skill(&self, skill: &SkillRecord) -> Result<(), Error> {
        let conn = self.conn.lock();
        let status_str = match skill.status {
            SkillStatus::Active => "ACTIVE",
            SkillStatus::Shadow => "SHADOW",
            SkillStatus::SuspendedDrift => "SUSPENDED_DRIFT",
            SkillStatus::Revoked => "REVOKED",
        };
        let now = crate::security::clocks::MonotonicClock::now_nanos() as i64;

        conn.execute(
            "INSERT INTO skills (id, version, origin_scope, status, commitment_hash, last_shadow_eval, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id, version) DO UPDATE SET
                status = excluded.status,
                last_shadow_eval = excluded.last_shadow_eval",
            params![
                skill.id,
                skill.version,
                skill.origin_scope,
                status_str,
                skill.commitment_hash,
                skill.last_shadow_eval,
                now,
            ],
        )
        .map_err(|e| Error::Internal(format!("failed to save skill: {e}")))?;
        Ok(())
    }

    pub fn list_skills(&self) -> Result<Vec<SkillRecord>, Error> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT id, version, origin_scope, status, commitment_hash, last_shadow_eval FROM skills ORDER BY id ASC, version DESC",
            )
            .map_err(|e| Error::Internal(format!("failed to prepare list skills query: {e}")))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| Error::Internal(format!("failed to query skills: {e}")))?;

        let mut out = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| Error::Internal(format!("failed to read skill row: {e}")))?
        {
            let id: String = row.get(0).unwrap_or_default();
            let version: String = row.get(1).unwrap_or_default();
            let origin_scope: String = row.get(2).unwrap_or_default();
            let status_raw: String = row.get(3).unwrap_or_default();
            let commitment_hash: String = row.get(4).unwrap_or_default();
            let last_shadow_eval: Option<String> = row.get(5).ok();

            let status = match status_raw.as_str() {
                "ACTIVE" => SkillStatus::Active,
                "SHADOW" => SkillStatus::Shadow,
                "SUSPENDED_DRIFT" => SkillStatus::SuspendedDrift,
                _ => SkillStatus::Revoked,
            };

            out.push(SkillRecord {
                id,
                version,
                origin_scope,
                status,
                commitment_hash,
                last_shadow_eval,
            });
        }
        Ok(out)
    }

    pub fn update_skill_status(
        &self,
        id: &str,
        version: &str,
        status: SkillStatus,
    ) -> Result<(), Error> {
        let conn = self.conn.lock();
        let status_str = match status {
            SkillStatus::Active => "ACTIVE",
            SkillStatus::Shadow => "SHADOW",
            SkillStatus::SuspendedDrift => "SUSPENDED_DRIFT",
            SkillStatus::Revoked => "REVOKED",
        };
        conn.execute(
            "UPDATE skills SET status = ?1 WHERE id = ?2 AND version = ?3",
            params![status_str, id, version],
        )
        .map_err(|e| Error::Internal(format!("failed to update skill status: {e}")))?;
        Ok(())
    }

    // --- Profiles ---

    pub fn save_profile(&self, profile: &BrowserProfile) -> Result<(), Error> {
        let conn = self.conn.lock();
        let (kind_str, origin, expires_at, cap) = match &profile.kind {
            ProfileKind::Ephemeral => ("EPHEMERAL", None, None, None),
            ProfileKind::Persistent {
                origin,
                expires_at,
                storage_cap_bytes,
            } => (
                "PERSISTENT",
                Some(origin.clone()),
                expires_at.clone(),
                *storage_cap_bytes,
            ),
        };

        conn.execute(
            "INSERT INTO profiles (id, kind, origin, expires_at, storage_cap_bytes, storage_used_bytes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                kind = excluded.kind,
                origin = excluded.origin,
                expires_at = excluded.expires_at,
                storage_cap_bytes = excluded.storage_cap_bytes,
                storage_used_bytes = excluded.storage_used_bytes",
            params![
                profile.id,
                kind_str,
                origin,
                expires_at,
                cap.map(|c| c as i64),
                profile.storage_used_bytes as i64,
            ],
        )
        .map_err(|e| Error::Internal(format!("failed to save profile: {e}")))?;
        Ok(())
    }

    pub fn list_profiles(&self) -> Result<Vec<BrowserProfile>, Error> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare("SELECT id, kind, origin, expires_at, storage_cap_bytes, storage_used_bytes FROM profiles")
            .map_err(|e| Error::Internal(format!("failed to prepare list profiles query: {e}")))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| Error::Internal(format!("failed to query profiles: {e}")))?;

        let mut out = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| Error::Internal(format!("failed to read profile row: {e}")))?
        {
            let id: String = row.get(0).unwrap_or_default();
            let kind_raw: String = row.get(1).unwrap_or_default();
            let origin: Option<String> = row.get(2).ok();
            let expires_at: Option<String> = row.get(3).ok();
            let cap: Option<i64> = row.get(4).ok();
            let storage_used_bytes: i64 = row.get(5).unwrap_or(0);

            let kind = if kind_raw == "PERSISTENT" {
                ProfileKind::Persistent {
                    origin: origin.unwrap_or_default(),
                    expires_at,
                    storage_cap_bytes: cap.map(|c| c as u64),
                }
            } else {
                ProfileKind::Ephemeral
            };

            out.push(BrowserProfile {
                id,
                kind,
                storage_used_bytes: storage_used_bytes as u64,
            });
        }
        Ok(out)
    }

    pub fn delete_profile(&self, id: &str) -> Result<bool, Error> {
        let conn = self.conn.lock();
        let rows = conn
            .execute("DELETE FROM profiles WHERE id = ?1", params![id])
            .map_err(|e| Error::Internal(format!("failed to delete profile: {e}")))?;
        Ok(rows > 0)
    }

    // --- Quarantined Downloads ---

    pub fn save_download(&self, dl: &QuarantinedDownload) -> Result<(), Error> {
        let conn = self.conn.lock();
        let state_str = match dl.verification_state {
            DownloadVerificationState::Quarantined => "QUARANTINED",
            DownloadVerificationState::Verified => "VERIFIED",
            DownloadVerificationState::Rejected => "REJECTED",
        };
        let now = crate::security::clocks::MonotonicClock::now_nanos() as i64;

        conn.execute(
            "INSERT INTO quarantined_downloads (id, file_path, size_bytes, declared_type, verification_state, quarantined_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                verification_state = excluded.verification_state",
            params![
                dl.id,
                dl.file_path,
                dl.size_bytes as i64,
                dl.declared_type,
                state_str,
                now,
            ],
        )
        .map_err(|e| Error::Internal(format!("failed to save quarantined download: {e}")))?;
        Ok(())
    }

    pub fn list_downloads(&self) -> Result<Vec<QuarantinedDownload>, Error> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare("SELECT id, file_path, size_bytes, declared_type, verification_state FROM quarantined_downloads ORDER BY quarantined_at DESC")
            .map_err(|e| Error::Internal(format!("failed to prepare list downloads query: {e}")))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| Error::Internal(format!("failed to query downloads: {e}")))?;

        let mut out = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| Error::Internal(format!("failed to read download row: {e}")))?
        {
            let id: String = row.get(0).unwrap_or_default();
            let file_path: String = row.get(1).unwrap_or_default();
            let size_bytes: i64 = row.get(2).unwrap_or(0);
            let declared_type: Option<String> = row.get(3).ok();
            let state_raw: String = row.get(4).unwrap_or_default();

            let verification_state = match state_raw.as_str() {
                "VERIFIED" => DownloadVerificationState::Verified,
                "REJECTED" => DownloadVerificationState::Rejected,
                _ => DownloadVerificationState::Quarantined,
            };

            out.push(QuarantinedDownload {
                id,
                file_path,
                size_bytes: size_bytes as u64,
                declared_type,
                verification_state,
            });
        }
        Ok(out)
    }

    pub fn update_download_verification(
        &self,
        id: &str,
        verification_state: DownloadVerificationState,
    ) -> Result<(), Error> {
        let conn = self.conn.lock();
        let state_str = match verification_state {
            DownloadVerificationState::Quarantined => "QUARANTINED",
            DownloadVerificationState::Verified => "VERIFIED",
            DownloadVerificationState::Rejected => "REJECTED",
        };
        conn.execute(
            "UPDATE quarantined_downloads SET verification_state = ?1 WHERE id = ?2",
            params![state_str, id],
        )
        .map_err(|e| Error::Internal(format!("failed to update download state: {e}")))?;
        Ok(())
    }

    // --- Settings ---

    pub fn get_settings(&self) -> Result<Option<Settings>, Error> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare("SELECT value FROM settings WHERE key = 'app_settings'")
            .map_err(|e| Error::Internal(format!("failed to prepare settings query: {e}")))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| Error::Internal(format!("failed to query settings: {e}")))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| Error::Internal(format!("failed to read settings row: {e}")))?
        {
            let val: String = row.get(0).unwrap_or_default();
            let settings: Settings = serde_json::from_str(&val)
                .map_err(|e| Error::Internal(format!("invalid stored settings json: {e}")))?;
            Ok(Some(settings))
        } else {
            Ok(None)
        }
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<(), Error> {
        let conn = self.conn.lock();
        let val = serde_json::to_string(settings)
            .map_err(|e| Error::Internal(format!("failed to serialize settings: {e}")))?;
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('app_settings', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![val],
        )
        .map_err(|e| Error::Internal(format!("failed to save settings: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory_and_run_migrations() {
        let store = SqliteStore::open_in_memory().unwrap();
        let tasks = store.list_tasks(10).unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn roundtrip_task_persistence() {
        let store = SqliteStore::open_in_memory().unwrap();
        let task = StoredTask {
            id: "t-123".into(),
            status: TaskState::Running,
            progress: TaskProgress {
                task_id: "t-123".into(),
                current_state: TaskState::Running,
                step_index: 3,
                total_steps: 11,
                epoch: 5,
                mode: ProgressMode::ModelReasoning,
            },
            preserved_state_summary: Some("testing".into()),
        };

        store.save_task(&task).unwrap();
        let loaded = store.get_task("t-123").unwrap().expect("task found");
        assert_eq!(loaded.id, "t-123");
        assert_eq!(loaded.progress.step_index, 3);
        assert_eq!(loaded.progress.epoch, 5);
        assert_eq!(loaded.preserved_state_summary.as_deref(), Some("testing"));
    }

    #[test]
    fn roundtrip_vault_item_persistence() {
        let store = SqliteStore::open_in_memory().unwrap();
        let item = VaultItemRef {
            id: "v-1".into(),
            origin: "https://example.com".into(),
            account_label: "user@example.com".into(),
            scope: crate::core::policy::scopes::CredentialScope::ExactOrigin {
                origin: "https://example.com".into(),
            },
            https_check_state: HttpsCheckState::Pass,
            idn_homograph_check_state: IdnHomographCheckState::Pass,
            last_used_task: None,
            redacted_secret: None,
        };
        let blind_index = BlindIndexValue { value: [42u8; 32] };
        let secret = b"encrypted_payload";
        let nonce = [1u8; 12];

        store
            .save_vault_item(&item, &blind_index, secret, &nonce)
            .unwrap();
        let items = store.list_vault_items().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "v-1");
        assert_eq!(items[0].origin, "https://example.com");

        let (enc, n) = store.get_vault_item_secret("v-1").unwrap().unwrap();
        assert_eq!(enc, secret);
        assert_eq!(n, nonce);

        assert!(store.delete_vault_item("v-1").unwrap());
        assert!(store.list_vault_items().unwrap().is_empty());
    }
}
