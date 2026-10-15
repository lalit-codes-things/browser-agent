# Contributing to Browser-Agent

Thank you for considering a contribution. Browser-Agent is a security-sensitive desktop browser
project. Correctness, privacy, accessibility, reproducibility, and truthful status reporting take
priority over feature count or demo quality.

## Before opening an issue

- Search existing issues and discussions for duplicates.
- For security vulnerabilities, **do not** open a public issue; follow `SECURITY.md`.
- For bugs, include the exact commit, OS/version, architecture, reproduction steps, expected
  behavior, observed behavior, logs with secrets removed, and whether the result is reproducible.
- For feature proposals, describe the user problem, scope, security/privacy/accessibility impact,
  alternatives considered, and measurable acceptance criteria.

## Non-negotiable project rules

1. The model proposes actions; trusted Rust policy authorizes them.
2. Page content is untrusted input and must never grant authority.
3. Do not add cloud inference, remote telemetry, automatic crash uploads, or vendor API-key
   requirements.
4. Do not use screenshots, fake pages, mock progress, or simulated success as substitutes for real
   browser behavior.
5. Missing or failed inference must not disable ordinary manual browsing.
6. Unknown or irreversible actions require the specified user handoff; autonomous payment submission
   is out of scope.
7. Do not weaken Chromium sandboxing, site isolation, origin boundaries, private-context separation,
   or credential isolation.
8. Never put secrets, raw credentials, cookies, authorization headers, or unredacted browsing data
   in logs, fixtures, screenshots, tests, or pull requests.
9. Requirements stay `UNVERIFIED` until the defined evidence has actually been collected. Code
   presence and a successful build alone do not prove behavioral or security requirements.
10. Follow the authoritative `docs/DESIGN-FINAL.md`. If it conflicts with code or older documents,
    explain the discrepancy rather than silently choosing a new direction.

## Development workflow

1. Start from a fresh checkout and inspect the pinned toolchain and lockfiles.
2. Create a focused branch from the current protected base branch.
3. Make the smallest coherent change that satisfies a stated issue or requirement.
4. Add or update tests and documentation in the same change.
5. Run the same checks used by CI. Do not regenerate lockfiles unless dependency changes are
   intentional.
6. Review the diff for secrets, unrelated formatting, generated files, license obligations, and
   changed security boundaries.
7. Open a pull request using the repository template and link the issue/requirement IDs.

Do not claim checks passed unless they were run. Paste the command and concise result, or link to
the CI run.

## Architecture and requirement changes

Architecture, security, privacy, egress, CEF bridge, vault/secure-fill, model runtime, packaging,
and update changes require:

- affected stable requirement IDs (`BA-AREA-NNN`);
- current behavior and proposed behavior;
- rationale and alternatives;
- threat-model and privacy impact;
- accessibility impact where relevant;
- tests and evidence artifacts;
- updates to `requirements/registry.yaml` and the requirement-to-evidence matrix;
- an Architecture Decision Record (ADR) when the specification requires one.

Never invent or renumber requirement IDs. Do not mark requirements `PASS` based on source inspection
alone. Use the repository's allowed status vocabulary.

## Security-sensitive areas

Changes touching any of the following require extra scrutiny and CODEOWNERS review:

- CEF lifecycle, native bridge, process boundaries, request contexts, or production debugging
  exposure;
- policy authorization, action schemas, origin/frame/epoch binding, or approval expiry;
- credential vault, secure fill, secret redaction, or persistence;
- egress controls, URL navigation, downloads, external protocol handling, or network policy;
- audit integrity, safe checkpoints, recovery, or irreversible/unknown actions;
- local inference IPC, model loading, model artifact verification, or resource hard stops;
- CI workflows, dependency allowlists, signing identities, release artifacts, and update
  verification.

A UI-only confirmation is not a substitute for authorization enforced by the trusted runtime.

## Coding expectations

- Keep changes small, explicit, typed, and testable.
- Prefer deterministic behavior and structured errors over implicit retries.
- Avoid silent fallback behavior.
- Use existing project formatters, linters, and patterns; do not add a second competing
  configuration.
- Add negative tests for denied actions and failure cases, not only happy-path tests.
- Keep local and CI checks aligned.
- Pin and review new dependencies. Explain why a dependency is necessary and document license
  implications.
- Avoid unrelated dependency upgrades, generated churn, and broad refactors in feature PRs.
- Do not introduce dead controls or UI states that do not map to real runtime behavior.

## Pull request checklist

- [ ] The change solves a defined problem and links the issue/requirement.
- [ ] The authoritative design/security contract is followed or explicitly amended.
- [ ] Tests cover success, failure, denial, and relevant race/stale-state cases.
- [ ] Relevant formatting, lint, typecheck, build, and test commands were run.
- [ ] No secrets or sensitive browsing data are included.
- [ ] Dependency, license, SBOM, and supply-chain implications were reviewed.
- [ ] Requirement registry and evidence matrix are updated when applicable.
- [ ] Documentation and error copy describe actual behavior.
- [ ] Accessibility and OS-specific behavior were considered.
- [ ] Platform-specific checks are clearly marked unrun if not executed.
- [ ] No requirement is marked `PASS` without qualifying evidence.

## Review and conduct

Be respectful, specific, and open to technical disagreement. Critique code and decisions, not
people. See `CODE_OF_CONDUCT.md`.
