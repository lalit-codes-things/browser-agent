# ADR-003: Browser display and manual takeover

- Status: Accepted
- Date: 2026-10-06
- Decision owners: Browser Agent maintainers

## Decision
Run a separate headful managed Chromium window beside the Tauri control console. Rust communicates with Chromium through the selected CDP transport. The Tauri webview does not embed the external Chromium process.

## Takeover
Manual takeover is a bounded authority transition. When takeover is required, the task enters a parked/intervention state, the console states what authority is transferred, and the user interacts with the managed Chromium window. A screenshot-only readout never satisfies takeover.

## Implementation requirements
- Keep Chromium and the Tauri console as separate windows and trust surfaces.
- Emit takeover, intervention, authority, epoch, frame, loader, and process-state events from the backend.
- Add screencast and input forwarding only where needed to support an explicit takeover path.
- Invalidate stale actions and approvals when takeover or page state changes.
- Preserve the C-153 authority boundary and require explicit resume after user intervention.
