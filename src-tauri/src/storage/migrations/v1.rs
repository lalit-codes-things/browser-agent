// Migrations.
//
// C-127: schema/key migrations are safe and release-gated.

pub mod v1;

pub fn current_version() -> u32 {
    1
}
