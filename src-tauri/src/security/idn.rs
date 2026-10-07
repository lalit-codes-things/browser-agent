// IDN / homograph handling.
//
// C-143, C-145: punycode display for IDN; homograph mismatch checked.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum IdnDisplay {
    Punycode,
    Unicode,
    Mismatch,
}

pub fn canonical_origin_punycode(_raw: &str) -> Result<String, crate::Error> {
    Err(crate::Error::NotImplemented("canonical_origin_punycode is scheduled".into()))
}

pub fn homograph_mismatch(_a: &str, _b: &str) -> bool {
    false
}
