# ADR-001: Product platform scope

- Status: Accepted
- Date: 2026-10-06
- Decision owners: Browser Agent maintainers

## Decision
Browser Agent v1 supports macOS and Windows desktop deployments. Linux remains a CI and build-validation target only and is not a supported desktop product platform.

## Constraints preserved
The macOS constraints remain authoritative for macOS behavior. Windows support is implemented through explicit platform abstractions and appended catalog constraints; existing C-IDs are never renumbered or silently reinterpreted.

## Implementation requirements
- Keep platform services behind traits for egress, authentication, key storage, app-data paths, memory pressure, process hardening, clipboard hygiene, quarantine, Chromium transport, inference, packaging, and shortcuts.
- Run CI validation on macOS, Windows, and Linux where toolchain dependencies permit.
- Do not claim Linux desktop support in release metadata.
- Add Windows-specific evidence before marking Windows constraints verified.

## Consequences
macOS remains the reference security envelope. Windows requires separate implementations for WFP, Windows Hello, DPAPI/Credential Manager, WebView2, process mitigations, clipboard metadata, Zone.Identifier quarantine, inherited-handle transport, and NSIS/MSI signing.
