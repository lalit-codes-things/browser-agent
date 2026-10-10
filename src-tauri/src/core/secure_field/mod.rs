// Secure-field handling.
//
// This module is the shared boundary between runtime secrets and any
// frontend-visible representation. It is intentionally NOT the UI layer.
// The UI layer consumes only the representations this module produces, so
// payment secrets, clipboard secrets, and QR payload contents never reach
// React/Tauri IPC/frontend serialization as plaintext.
//
// Boundaries enforced here:
//   - payment-input region classification + redacted representation;
//   - agent clipboard prohibition (agent cannot copy secrets to pasteboard);
//   - user-initiated clipboard copy path (concealed/transient metadata where
//     permitted, with explicit platform-limitation note);
//   - QR region tracking, pixel-region hashing, epoch/frame binding, change
//     invalidation, handoff state, external-auth waiting state.
//
// UPI QR: the production agent does NOT decode QR images or parse QR
// payloads. That is enforced here and by the intended architecture; any
// reachable QR-decode path is a defect to remove.

pub mod clipboard;
pub mod qr_region;
pub mod sensitive_field;

pub use clipboard::*;
pub use qr_region::*;
pub use sensitive_field::*;
