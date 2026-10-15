# Maintainer pre-publish notes

This file is internal preparation guidance; remove it from the public repository if desired after
completing the checklist.

## Do not publish these placeholders unchanged

- `@MAINTAINER` in `.github/CODEOWNERS`
- `OWNER/REPO` in GitHub issue templates
- the license-pending statement until the license decision is complete
- any workflow paths or commands that do not match the real repository tree

## Before merging this pack

1. Compare it against the actual repository and keep existing equivalent configuration instead of
   creating duplicate competing files.
2. Merge documentation without overwriting authoritative design/security documents.
3. Verify each referenced path and command exists.
4. Confirm `.gitignore` does not hide required source or tracked lockfiles.
5. Inspect all workflows for permissions, pinned actions, path assumptions, and correct native
   runner setup.
6. Ensure `cargo-deny`, `cargo-audit`, and SBOM generation actually run and fail the job when
   required.
7. Run YAML/Markdown/JSON validation.
8. Check the staged diff for secrets and accidental binaries.
9. Resolve licensing before describing the project as open source.
10. Update README status from actual evidence on the final commit.

## CI workflow policy

Do not add a generic workflow that pretends to build CEF on every platform if the repository lacks
the native project files or pinned CEF setup. The existing workflow should be edited to match the
actual build graph. CI config existence is not evidence that a check passed.
