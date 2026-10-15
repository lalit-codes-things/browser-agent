# Browser-Agent

**A browser that can act for you.**

Browser-Agent is a local-first desktop browser project combining a Rust application core, a
React/TypeScript interface, an embedded Chromium runtime through CEF, and an optional local
inference subsystem. Its design prioritizes user control, explicit authorization, evidence-based
verification, and honest reporting of unsupported or unverified behavior.

> **Development status: pre-release / not yet ready for general use.**
>
> The project specification is more complete than the implementation. The CEF native host and
> cross-platform composition proof remain outstanding; llama.cpp is not yet wired; and supply-chain
> checks must be verified as enforced in CI. Do not interpret the architecture or feature
> descriptions below as claims that these capabilities currently work.

## Project status

| Area                                | Current status                                                                                                     |
| ----------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| Product/architecture specification  | Defined in `docs/DESIGN-FINAL.md` (maintainer should place the authoritative spec there)                           |
| Rust core                           | In active development; build/test status depends on the current commit                                             |
| Real CEF page rendering/native host | Not yet complete; real-page rendering and macOS + Windows composition proof remain outstanding                     |
| Local inference                     | llama.cpp not wired; readiness must remain `NOT_INSTALLED` until real checks pass                                  |
| Supply-chain security               | `deny.toml` exists, but `cargo-deny`, `cargo-audit`, and SBOM enforcement must be confirmed in CI                  |
| Requirement registry                | Must remain evidence-backed; no requirement should be marked `PASS` without recorded platform/commit/test evidence |
| Release packages/signing            | Not claimed                                                                                                        |

The status above is a maintainer-provided snapshot. Check the issue tracker and CI runs for changes
after this document was written.

## Principles

- **Local-first:** no required cloud inference endpoint or vendor API key.
- **Real browser, not a mock:** the product must render and interact with the actual managed CEF
  page.
- **Policy is independent of the model:** model output is a proposal, not authorization.
- **User control:** pause, cancellation, takeover, and consequential-action handoff are first-class
  requirements.
- **Evidence over claims:** attempted actions and verified outcomes are different states.
- **Privacy by default:** no unsolicited telemetry or automatic diagnostic uploads; sensitive
  operations fail closed.
- **Manual browsing remains independent:** missing inference must not disable ordinary browsing.
- **No autonomous payment submission:** the agent may prepare a workflow and hand control to the
  user for payment or another consequential action.

## Intended platforms and architecture

- macOS on Apple silicon
- Windows x64
- Rust core and policy/runtime
- React + TypeScript UI
- Chromium Embedded Framework (CEF) native rendering host with a narrow audited C++/Rust bridge
- llama.cpp as the planned local inference engine

The native browser host and supported-platform composition must pass the project's feasibility and
release gates before these platform claims are treated as validated support.

## What works today?

Do not infer feature availability from the design specification alone. For the authoritative
implementation state:

1. Inspect the current CI status and commit.
2. Review open issues tagged `status: blocked` or `status: needs evidence`.
3. Check the requirements registry and evidence matrix.
4. Run the applicable build and test commands documented in the repository.
5. Confirm that a real CEF page—not a screenshot or placeholder—is displayed before considering
   browser rendering implemented.

If these status artifacts are absent or stale, treat the feature as **unverified**.

## Repository map

The repository may evolve while implementation is underway. Keep this section synchronized with the
actual tree.

- `crates/` — Rust crates, if present
- `ui/` — React/TypeScript application, if present
- `native/` — CEF/C++ host and bridge, if present
- `requirements/registry.yaml` — canonical stable requirement inventory
- `docs/DESIGN-FINAL.md` — authoritative product/architecture/quality specification
- `docs/THREAT_MODEL.md` — threat model
- `.github/workflows/` — CI, security, and release workflows

Only paths that exist in the checked-out commit are implementation evidence.

## Development setup

### Prerequisites

Install only the toolchains required by the current repository. The authoritative specification
calls for a pinned Rust toolchain, Node.js for the UI, and platform-specific native build tools for
CEF. CEF binaries and resources must be obtained from a trusted, documented source and verified
against the repository's pinned version and checksums.

Before starting, inspect:

- `rust-toolchain.toml`
- root and crate `Cargo.toml` files
- `package.json` / `ui/package.json`
- lockfiles
- the native build documentation
- the CEF version, download source, hashes, and license notices

Do not install arbitrary global dependencies or silently update lockfiles.

### Build and test

Commands depend on the current repository layout. Use the checked-in scripts and CI workflow as the
source of truth. Typical checks, where the corresponding manifests and targets exist, include:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --locked
cargo deny check
cargo audit
npm ci
npm run typecheck
npm run lint
npm test
npm run build
```

Do not run or document a command as supported unless its corresponding manifest/script exists.
Native CEF builds and runtime tests must be run on their supported operating systems; a Rust/UI-only
CI pass does not prove native page rendering or cross-platform composition.

## Security and privacy

Please read [SECURITY.md](SECURITY.md) before reporting a vulnerability.

- Never commit API keys, tokens, passwords, private browsing data, cookies, model weights, signing
  credentials, or unredacted diagnostic bundles.
- Never weaken sandboxing, origin isolation, credential boundaries, or policy checks to make a demo
  pass.
- Do not add a cloud inference fallback.
- Do not expose a production CDP endpoint.
- Keep private-window state isolated and ephemeral.
- Do not mark a requirement `PASS` without the required executed evidence.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md), follow the pull request template, and review the design
specification before proposing architecture or UX changes. Security-sensitive changes require
explicit threat-model and test updates.

## License

**License selection is pending.** This repository should not be published as an open-source project
until the maintainer chooses a license, adds the complete license text as `LICENSE`, and verifies
that the chosen license is compatible with every dependency, CEF/Chromium component, font, native
library, and other bundled asset. See `docs/OPEN_SOURCE_RELEASE_CHECKLIST.md`.

Third-party components retain their own licenses. See `THIRD_PARTY_NOTICES.md`; that inventory must
be completed from the actual shipped dependency set before distributing binaries.
