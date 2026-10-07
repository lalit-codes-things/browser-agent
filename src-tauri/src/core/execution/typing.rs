// Hybrid typing.
//
// C-88: hybrid typing uses per-key events for final chunks of
//        framework-controlled inputs and insertText + synthetic events for
//        bulk; model cannot control JavaScript.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingInstruction {
    pub target_reference: String,
    pub text: String,
    pub final_chunk: bool,
}

pub struct TypingEngine;

impl TypingEngine {
    pub fn type_text(_instruction: TypingInstruction) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented("TypingEngine::type_text is scheduled".into()))
    }
}
