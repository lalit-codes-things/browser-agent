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
// Payment/authorization clipboard and QR operations are routed through
// typed command schemas, not through generic React clipboard or model
// context.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitTaskRequest {
    pub task_text: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitTaskResult {
    pub task_id: String,
}

// Payment command schemas are added where the production path is wired.
// No payment secret is ever carried in an IPC command.

#[derive(Debug, Deserialize, Serialize)]
pub struct BeginPaymentSubmissionRequest {
    pub task_id: String,
    pub payable_json: String,
    pub commitment_hash: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaymentSubmissionResult {
    pub durable_id: String,
    pub durable_state: String,
    pub payment_commitment_hash: String,
}
