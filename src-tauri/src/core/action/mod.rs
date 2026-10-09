// Generic action authorization and durable action state.
//
// Payment-specific architecture has been removed from the core. This module
// preserves the useful generic security mechanisms: canonical action
// commitments, authorization capabilities bound to commitment + epoch, and
// durable action state used for recovery and human handoff.
//
// The browser must stop and wait for the user at payment/checkout. This
// module models generic consequential-action authorization and recovery; it
// does not implement payment automation.

pub mod authz;
pub mod commitment;
pub mod durable;
pub mod idempotency;

pub use commitment::{ActionAuthorizationCapability, OperationType};
pub use authz::{authorize_action_commitment, verify_action_authorization_capability};

