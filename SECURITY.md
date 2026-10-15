# Security Policy

## Project status and scope

Browser-Agent is a pre-release, local-first desktop browser project. The architecture and security
requirements are not proof that every control is implemented or validated. Please identify the exact
commit and platform when reporting a finding.

## Reporting a vulnerability

**Do not disclose exploitable vulnerabilities in public issues, pull requests, or discussions.**

Until a private security advisory contact is configured, use GitHub's private vulnerability
reporting feature if enabled for this repository. If it is unavailable, contact the repository
maintainer through a private channel listed on the repository's GitHub profile. Do not include
secrets or real user browsing data in the initial report.

Include, where safe:

- affected commit/version and platform;
- impact and attacker prerequisites;
- minimal reproduction steps or a proof of concept;
- expected versus actual behavior;
- relevant logs with credentials, tokens, cookies, personal data, and private URLs removed;
- any known mitigation.

Do not test against other people's accounts, production websites, or systems without authorization.
Do not exfiltrate real secrets or publish exploit details before coordinated remediation.

## Response expectations

The maintainer should acknowledge receipt, assess severity and reproducibility, coordinate a fix,
and agree on disclosure timing with the reporter. Do not promise a fixed response time until the
project has a staffed response process.

## Security-critical boundaries

Reports are especially important for:

- page/prompt injection that changes authorization;
- actions executed without current Rust policy authorization;
- stale target, origin, frame, or observation-epoch reuse;
- cross-origin or cross-context credential/data leakage;
- private-window data persistence or cross-profile access;
- CEF native bridge privilege exposure or production CDP exposure;
- unsafe model-runner IPC, model artifact tampering, or cloud inference fallback;
- audit tampering, secret leakage, unsafe recovery, or blind replay after ambiguous side effects;
- sandbox/site-isolation weakening;
- dependency, build, signing, update, or supply-chain compromise.

## Supported versions

There are no stable production releases at this time. Report the exact commit. Once releases exist,
the maintainer should define a supported-version window here.

## Security posture

- Do not assume a check is enforced because a config file exists.
- Do not treat an SBOM or clean scanner output as proof that an artifact is safe.
- Critical authorization/privacy bypasses block release.
- Unknown license, unverified binary origin, or artifact hash mismatch blocks packaging.
- Diagnostic exports must be explicitly initiated, previewed, and redacted; they are never uploaded
  automatically.
