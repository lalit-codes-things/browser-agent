// Typed IPC schemas.
//
// These mirror the Rust event enums in src-tauri/src/ipc/events.rs.
// Frontend never invents these values; they come from the backend.

export type TaskStatus =
  | "PENDING"
  | "RUNNING"
  | "VERIFIED"
  | "LIKELY_SUCCESS"
  | "UNKNOWN"
  | "FAILED"
  | "ABORTED"
  | "PARKED";

export type ModelResidency = "LOADED" | "PARKED" | "UNLOADED";

export type VerificationOutcome =
  | "VERIFIED_SUCCESS"
  | "LIKELY_SUCCESS"
  | "UNKNOWN"
  | "LIKELY_FAILURE"
  | "VERIFIED_FAILURE";

export interface SubmitTaskRequest {
  task_text: string;
}

export interface SubmitTaskResult {
  task_id: string;
}
