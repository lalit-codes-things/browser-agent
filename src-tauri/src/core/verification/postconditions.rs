// Postconditions.
//
// C-94: secure-fill verification relies on input-state mutation plus form
//        transition and never reads the secret value back.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostconditionCheck {
    pub name: String,
    pub passed: bool,
    pub detail: Option<String>,
}
