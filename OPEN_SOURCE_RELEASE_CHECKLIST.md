# Open-source publication checklist

This checklist is deliberately conservative. A repository being public does not mean its
implementation is release-ready.

## A. Repository identity

- [ ] Set the correct GitHub owner/repository in issue-template links.
- [ ] Replace every `@MAINTAINER` placeholder in `.github/CODEOWNERS`.
- [ ] Add a real security contact/private vulnerability reporting route.
- [ ] Confirm the repository name, description, topics, homepage, and support channels.
- [ ] Decide whether to enable Discussions and private vulnerability reporting.
- [ ] Add a project logo only if the asset and its license are verified.

## B. Legal and licensing — required before public release

- [ ] Choose an explicit source license after considering intended reuse, patent terms, copyleft,
      and future distribution.
- [ ] Add the complete selected license text as `LICENSE`; do not publish a placeholder as a
      license.
- [ ] Confirm copyright-holder attribution and years.
- [ ] Audit every direct/transitive dependency, CEF/Chromium component, llama.cpp component, font,
      native library, and bundled asset.
- [ ] Complete `THIRD_PARTY_NOTICES.md` and include required license texts/notices.
- [ ] Review whether model weights are separate downloads and verify their exact licenses and
      hashes.
- [ ] Review trademarks and third-party branding.

## C. Implementation truthfulness

- [ ] README status matches the actual commit and current CI evidence.
- [ ] CEF native host exists and displays a real managed page.
- [ ] macOS Apple-silicon and Windows x64 composition tests have executed and passed, or the project
      remains explicitly marked pre-release/blocked.
- [ ] llama.cpp readiness is derived from real install, integrity, compatibility, and runner health
      checks; no fabricated progress.
- [ ] Manual browsing works without a model.
- [ ] No screenshot/mock/placeholder is represented as real browser rendering.
- [ ] Requirements registry and evidence matrix contain accurate statuses and links.

## D. CI and supply chain

- [ ] Rust format, lint, test, and locked-build jobs execute.
- [ ] Frontend format/lint/typecheck/test/build jobs execute where the UI exists.
- [ ] Native CEF/C++ jobs run on supported operating systems.
- [ ] `cargo-deny` is installed and enforced; `deny.toml` is not merely present.
- [ ] `cargo-audit` is installed and enforced.
- [ ] SBOM is generated from actual build/dependency inputs and retained as an artifact.
- [ ] Secret scanning, dependency review, and static analysis are configured.
- [ ] GitHub Actions are pinned to reviewed full commit SHAs for production workflows.
- [ ] CI checks are required by branch protection and cannot be bypassed by `continue-on-error`.
- [ ] Lockfiles are committed and CI uses frozen/locked dependency resolution.
- [ ] CEF, Chromium, native libraries, and model artifacts have pinned source/version/hash records.

## E. Privacy and security

- [ ] No secrets, model weights, browser profiles, crash dumps, private logs, or local credentials
      are tracked.
- [ ] Threat model is present and reflects current trust boundaries.
- [ ] Production CDP endpoint is disabled.
- [ ] Browser UI bridge is restricted to the exact first-party origin and approved frames.
- [ ] Private contexts are ephemeral and isolation tests exist.
- [ ] Audit integrity and failure behavior are tested.
- [ ] Diagnostic export is explicit, previewed, and redacted.
- [ ] No telemetry or automatic crash upload is present unless a reviewed spec amendment explicitly
      changes this.
- [ ] Security policy and responsible disclosure route work.

## F. Community and operations

- [ ] CONTRIBUTING, CODE_OF_CONDUCT, SUPPORT, GOVERNANCE, SECURITY, and issue/PR templates are
      present.
- [ ] Maintainer response expectations are realistic.
- [ ] CODEOWNERS placeholders are resolved.
- [ ] Release tags are protected and signed according to the release process.
- [ ] Changelog reflects landed changes only.
- [ ] Known limitations are documented.

## G. Distribution

- [ ] macOS app and DMG signing/notarization are proven on a release build.
- [ ] Windows installer signing is proven on a release build.
- [ ] Checksums, SBOM, notices, provenance, and test/security summaries are published.
- [ ] Model weights are distributed separately and installed explicitly with integrity verification.
- [ ] Rollback and uninstall behavior are tested and documented.

## Current known blockers

At the time this checklist was drafted:

1. The CEF native host and real-page rendering proof are outstanding.
2. macOS + Windows composition proof is outstanding.
3. llama.cpp is not wired; readiness must remain `NOT_INSTALLED`.
4. `cargo-deny`, `cargo-audit`, and SBOM CI enforcement must be verified/implemented.
5. `requirements/registry.yaml` must not be marked passing without executed platform/commit
   evidence.
6. License selection and full third-party inventory remain maintainer decisions.
