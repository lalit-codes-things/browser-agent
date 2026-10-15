# Governance

## Current model

Browser-Agent is currently maintainer-led. The repository maintainer is accountable for architecture
decisions, security triage, releases, and accepting contributions. `CODEOWNERS` identifies the areas
requiring explicit review; it does not itself guarantee independent review when only one maintainer
is available.

## Decision process

- Small implementation changes may be decided in pull-request review.
- Product, architecture, security-boundary, privacy, or release-policy changes require written
  rationale and the applicable ADR/specification updates.
- Stable requirement IDs are never reused or renumbered.
- Accepted deviations must identify the affected requirement, rationale, owner, evidence, and
  review/expiry conditions where applicable.
- Maintainers must not mark requirements as passing without the required evidence.

## Becoming a maintainer

Maintainers should demonstrate sustained, high-quality contributions, sound security judgment,
respectful collaboration, and familiarity with the product specification. The current maintainer
makes appointments and documents scope of responsibility in `CODEOWNERS`.

## Releases

Releases are made only from reviewed protected tags/commits after the applicable platform, security,
privacy, dependency, license, and packaging gates pass. A green UI or Rust job alone is not a
release qualification.
