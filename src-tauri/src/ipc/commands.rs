// Typed IPC commands.
//
// Every command must have:
//  - explicit type
//  - schema
//  - authorization expectation
//  - error behavior
//  - test coverage
//
// We keep this module small and explicit. No arbitrary command blobs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitTaskRequest {
    pub task_text: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitTaskResult {
    pub task_id: String,
}
