// Browser-facing generic action types.
//
// The browser subsystem uses the same generic action commitment / capability
// model as the core policy and execution engines. This module is a re-export
// boundary, not a duplicate implementation.

pub use crate::core::action::{
    ActionAuthorizationCapability, OperationType, authorize_action_commitment,
    verify_action_authorization_capability,
};
pub use crate::core::action::commitment::ActionCommitment;
