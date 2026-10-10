// Migrations.
//
// C-127: schema/key migrations are safe and release-gated.
// v1 schema DDL is delivered with the storage integration (SqliteStore).

pub fn current_version() -> u32 {
    1
}

pub const V1_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS _schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    status TEXT NOT NULL,
    step_index INTEGER NOT NULL,
    total_steps INTEGER NOT NULL,
    epoch INTEGER NOT NULL,
    mode TEXT NOT NULL,
    summary TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_records (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    segment_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL,
    hmac_link TEXT NOT NULL,
    segment_hmac TEXT NOT NULL,
    timestamp TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS vault_items (
    id TEXT PRIMARY KEY,
    blind_index BLOB NOT NULL,
    origin TEXT NOT NULL,
    account_label TEXT NOT NULL,
    scope TEXT NOT NULL,
    https_check_state TEXT NOT NULL,
    idn_check_state TEXT NOT NULL,
    encrypted_secret BLOB NOT NULL,
    nonce BLOB NOT NULL,
    last_used_task TEXT,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS skills (
    id TEXT NOT NULL,
    version TEXT NOT NULL,
    origin_scope TEXT NOT NULL,
    status TEXT NOT NULL,
    commitment_hash TEXT NOT NULL,
    last_shadow_eval TEXT,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (id, version)
);

CREATE TABLE IF NOT EXISTS quarantined_downloads (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    declared_type TEXT,
    verification_state TEXT NOT NULL,
    quarantined_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS profiles (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    origin TEXT,
    expires_at TEXT,
    storage_cap_bytes INTEGER,
    storage_used_bytes INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;
