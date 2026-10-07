// Migrations.
//
// C-127: schema/key migrations are safe and release-gated.
// v1 schema DDL is delivered with the storage integration (SqliteStore).

pub fn current_version() -> u32 {
    1
}
