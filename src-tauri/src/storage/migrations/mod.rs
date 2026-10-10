// Migrations.
//
// C-127: schema/key migrations are safe and release-gated.

pub mod v1;

use crate::error::Error;
use rusqlite::Connection;

pub fn current_version() -> u32 {
    1
}

pub fn run_migrations(conn: &Connection) -> Result<(), Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )
    .map_err(|e| Error::Internal(format!("failed to initialize migrations table: {e}")))?;

    let mut stmt = conn
        .prepare("SELECT MAX(version) FROM _schema_migrations")
        .map_err(|e| Error::Internal(format!("failed to check schema version: {e}")))?;
    let version: Option<u32> = stmt
        .query_row([], |row| row.get(0))
        .map_err(|e| Error::Internal(format!("failed to read schema version: {e}")))?;
    let applied_version = version.unwrap_or(0);

    if applied_version < 1 {
        conn.execute_batch(v1::V1_SCHEMA_SQL)
            .map_err(|e| Error::Internal(format!("failed to apply v1 migration: {e}")))?;
        conn.execute(
            "INSERT INTO _schema_migrations (version, applied_at) VALUES (1, datetime('now'))",
            [],
        )
        .map_err(|e| Error::Internal(format!("failed to record v1 migration: {e}")))?;
    }

    Ok(())
}
