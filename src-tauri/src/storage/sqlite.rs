// SQLite runtime store.
//
// Persistent local store for runtime state. Schema is release-gated (C-127).

pub struct SqliteStore;

impl SqliteStore {
    pub fn open(_path: &str) -> Result<Self, crate::Error> {
        Err(crate::Error::NotImplemented("SqliteStore::open is scheduled".into()))
    }
}
