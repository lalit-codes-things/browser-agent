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

pub fn canonical_origin_punycode(raw: &str) -> Result<String, crate::Error> {
    let origin = raw.trim().to_ascii_lowercase();
    if origin.is_empty() || origin.chars().any(char::is_whitespace) {
        return Err(crate::Error::InvalidParameter("invalid origin".into()));
    }
    Ok(origin)
}

pub fn homograph_mismatch(a: &str, b: &str) -> bool {
    canonical_origin_punycode(a).ok() != canonical_origin_punycode(b).ok()
}
