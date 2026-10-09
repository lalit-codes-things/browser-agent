# ADR-006: Chromium pinning and CDP transport

- Status: Accepted pending transport verification
- Date: 2026-10-06
- Decision owners: Browser Agent maintainers

## Decision
Use a pinned Chrome for Testing or Chromium build with a recorded version and SHA-256 digest. Chromium is re-pinned only through a regression-gated pipeline. No unpinned system browser is used for the managed runtime.

The required transport is `--remote-debugging-pipe`, with no TCP debugging endpoint. Before selecting `chromiumoxide`, verify that the pinned implementation supports the pipe transport and inherited-handle behavior on macOS and Windows.

## Transport gate
If the selected library only supports a WebSocket/TCP endpoint, do not silently weaken C-04. Either implement a dedicated inherited-file-descriptor/handle CDP transport or record and approve a constraint-amending ADR before changing the transport.

## Implementation requirements
- Pin Chromium version, platform artifact, and SHA-256 digest.
- Keep sandbox and site isolation enabled.
- Disable extensions, sync, component updates, pings, QUIC bypass, and unnecessary permissions as required by the catalog.
- Add a negative test proving no TCP debugging endpoint is exposed.
- Record transport compatibility evidence for macOS and Windows before release.
