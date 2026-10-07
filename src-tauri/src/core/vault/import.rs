// Vault import.
//
// Import paths never expose raw secrets into model context or frontend state.

pub struct VaultImport;

impl VaultImport {
    pub fn import_item(_ref: crate::core::vault::items::VaultItemRef) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented("VaultImport::import_item is scheduled".into()))
    }
}
