// Slice contracts.
//
// This module holds the first hostile mock-site contract and the minimal
// runtime expectations the hostile suite must satisfy. It is intentionally
// separate from the policy/engine code so the hostile suite can be expanded
// without polluting the trusted policy tree.

pub mod mock_harness;
pub mod mock_site;
