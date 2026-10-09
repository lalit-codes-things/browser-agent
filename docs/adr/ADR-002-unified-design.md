# ADR-002: Unified product design

- Status: Accepted
- Date: 2026-10-06
- Decision owners: Browser Agent maintainers

## Decision
Merge DESIGN-2's browser-shell information architecture with DESIGN.md's trusted inspector, authorization interlock, task ledger, and evidence drawer. DESIGN.md is the binding design specification after this merge.

## Conflict resolution
- The browser shell is a supervisory projection, not a second trust boundary.
- The trusted inspector remains the authoritative surface for policy, authority, evidence, and audit state.
- No icon-only controls are introduced; text labels and keyboard semantics remain required.
- Browser shortcuts remain optional convenience controls, never command-style authority controls.
- Confirmation controls use explicit deny/stop and approval wording required by the binding coding rules, not generic Confirm/Cancel labels.
- The browser viewport is a controlled visual surface; the screenshot/readout and trusted inspector cannot authorize actions.

## Implementation requirements
Every security-relevant value is backend-emitted, page-derived strings are visibly quarantined, and the design must preserve the interlock, ledger, evidence drawer, hard-stop, and intervention surfaces.
