# ADR-005: Model distribution and licensing

- Status: Accepted
- Date: 2026-10-06
- Decision owners: Browser Agent maintainers

## Decision
The local model is distributed as a signed release asset fetched during installation or build preparation. It is never fetched at runtime, committed to Git, or distributed through Git LFS.

## Integrity and licensing
Each release asset has a pinned version, SHA-256 digest, signature, source revision, license record, and NOTICE entry. The model's exact license must be verified against the selected upstream release before publication; no license assumption is sufficient for release approval.

## Implementation requirements
- Keep runtime model paths platform-derived and outside the repository checkout.
- Verify the signed asset before installation and before every load.
- Hash and load the same protected file handle or private verified copy to avoid a path-swap window.
- Make model download failures explicit and fail closed.
- Keep model output untrusted and schema/grammar validated.
