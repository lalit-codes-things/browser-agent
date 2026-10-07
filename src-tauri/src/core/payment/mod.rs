// Payment domain.
//
// Payments are a HIGH_STAKES subclass of financial action. The production
// flow is canonical commitment -> POLICY/BIOMETRIC_CONFIRM authorization ->
// durable durable-submission-state -> external flow -> independent
// verification (C-14, C-15, C-22, C-39).
//
// The model never sees payment secrets. It reasons over payment state and
// task authority, not over secrets, QR payloads, or CVV/PAN/UPI PIN.

pub mod payment;
pub mod commitment;
pub mod durability;
pub mod recovery;
pub mod idempotency;

pub use payment::*;
pub use commitment::*;
pub use durability::*;
pub use recovery::*;
pub use idempotency::*;
