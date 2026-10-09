# ADR-004: Payments in v1

- Status: Accepted
- Date: 2026-10-06
- Decision owners: Browser Agent maintainers

## Decision
Automated card, UPI, QR, and payment submission are frozen out of the v1 execution path. Payment-related tasks stop before submission and hand control to the user through the intervention surface.

## Safety boundary
The agent may observe and display redacted payment state needed to explain a stop, but it must not enter, transmit, confirm, retry, or verify a payment as an autonomous action. No frontend IPC request may provide an amount, recipient, payable object, or commitment for submission.

## Implementation requirements
- Keep payment schemas only for backend-owned, redacted state and future migration compatibility.
- Remove or disable automated payment commands from the active capability set.
- Treat payment and financial actions as hard-stop/intervention outcomes.
- Preserve secret redaction, QR non-decoding, and independent verification boundaries.
- Add a future ADR before re-enabling any payment adapter.
