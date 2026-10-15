# Execution evidence

- Recorded: 2026-10-14T20:22:14Z
- Commit: `182e8ab5e425e103ae26dbd66f671744d5c1db0a`
- Working tree: modified (digest of tracked changes `f48355cd20fdaab4a18e2e9133fc20361f43509739f520c69b1701e841b2772f`, 75 untracked file(s))
- Tested content digest: `c77da8fab7a9defdccae1a341943fae1fc3d6436ceedc2e56c12fad0b647de87` over 120 tracked and untracked source files
- Platform: Darwin-arm64
- OS: macOS 27.0
- rustc: rustc 1.99.0 (b940084d7 2026-09-28)
- cargo: cargo 1.99.0 (5f94df478 2026-08-27)
- node: v24.21.0
- npm: 11.19.0

## Checks that ran

| Check | Command | Outcome | Exit | Output |
| --- | --- | --- | --- | --- |
| `rust-format` | `cargo fmt --all -- --check` | passed | 0 | `logs/rust-format.log` |
| `rust-clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | passed | 0 | `logs/rust-clippy.log` |
| `rust-tests` | `cargo test --workspace` | passed | 0 | `logs/rust-tests.log` |
| `core-binary` | `cargo build -p ba-runtime --bin ba-core` | passed | 0 | `logs/core-binary.log` |
| `ui-format` | `npm run format:check` | failed | 1 | `logs/ui-format.log` |
| `ui-lint` | `npm run lint` | passed | 0 | `logs/ui-lint.log` |
| `ui-types` | `npm run typecheck` | passed | 0 | `logs/ui-types.log` |
| `ui-tests` | `npm test` | passed | 0 | `logs/ui-tests.log` |
| `ui-build` | `npm run build` | passed | 0 | `logs/ui-build.log` |
| `supply-chain` | `./scripts/supply-chain.sh` | passed | 0 | `logs/supply-chain.log` |

A `passed` row means the command exited 0 on this platform and commit. The
complete output of every check is kept in `logs/` beside this file, so a
reader can inspect what actually ran rather than trusting the summary.

## Proofs not run

| Proof | Required by | Why it is absent |
| --- | --- | --- |
| CEF native view composition, focus and input routing | [BA-BROWSER-001], [BA-CEF-001], Phase 0/1 | The native CEF host does not exist in this checkout yet (ADR-010). No page is rendered, and the interface reports that absence instead of simulating one. |
| Window resize, multiple monitors, high-DPI, full-screen behavior | [BA-BROWSER-001] | Requires the native host and a display; cannot be exercised by the automated suites here. |
| Windows x64 results | Section 20.3, Section 21 | This record was produced on Darwin-arm64 only. Windows evidence must be recorded separately on that platform. |
| Packaged macOS/Windows installers, signing and notarization | Phase 8, Section 20.3 | No packaging pipeline exists yet. |
| Local model install and inference | Section 16, Phase 5 | No model artifact is registered in this build and no runner is integrated, so readiness stays `NOT_INSTALLED`; no progress is simulated. |
| CDP-disabled production inspection | Section 11.5 | There is no production browser binary to inspect yet. |

A check that passed here is not evidence for any row above: absent evidence
stays absent until it is produced, outside this document.
