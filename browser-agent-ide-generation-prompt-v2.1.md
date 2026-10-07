# IDE MASTER GENERATION PROMPT — Browser Agent / Autonomous Local Browser Runtime

You are the implementation IDE agent for this repository.

The repository will contain `catalog/constraints.yaml` from the Constraint Catalog v2.1 supplied separately by the user. Treat that file as the machine-readable source of truth for specification constraints. Do not rewrite, renumber, reinterpret, weaken, or silently expand catalog constraints.

The companion Markdown catalog is generated output and must not become the source of truth.

## 1. Primary objective

Build the complete repository architecture described by the Revision 2 specification and Catalog v2.1. This is NOT an MVP task. Generate the full production-oriented repository structure, Rust/Tauri core, browser runtime, security/policy layers, local inference pipeline, verification/audit stack, skills system, benchmark/security harness, release tooling, and thin React/TypeScript UI.

Implement the system phase-by-phase, but create the complete final tree now so later phases have stable module boundaries.

Do not fake implementation with placeholder success paths. Where a capability is intentionally scheduled for a later phase, create its correct interface, types, tests, and explicit `NOT_IMPLEMENTED_YET`/gated behavior rather than pretending it works.

## 2. Source-of-truth rules

1. `catalog/constraints.yaml` is authoritative for constraint IDs, specification status, verification status, named Revision-2 mappings, parameters, dependencies, amendments, and supersession.
2. `catalog/constraint-catalog-v2.1.md` is generated documentation only.
3. Every security-sensitive implementation decision must reference affected C-IDs in code comments, ADRs, tests, or PR metadata where appropriate.
4. Never invent a numeric value for a catalog parameter whose `value_defined: false`.
5. Never mark a constraint VERIFIED merely because code exists. Verification status changes only from executed evidence.
6. `PROPOSED`/Phase-8–10 expansion items must not be treated as Phase-0–7 release-blocking requirements.
7. Preserve stable IDs. Never renumber existing C-IDs.
8. Use `supersedes`, `amends`, and `depends_on` relationships from the catalog instead of duplicating policy rules in prose.

## 3. Repository tree to generate

```text
browser-agent/
├── README.md
├── LICENSE
├── .gitignore
├── .gitattributes                         # *.gguf Git LFS rules
├── Cargo.toml                              # workspace
├── Cargo.lock
├── package.json                            # frontend/tooling workspace
├── pnpm-lock.yaml OR package-manager lockfile actually chosen
├── rust-toolchain.toml
├── deny.toml / cargo-audit configuration
│
├── catalog/
│   ├── constraints.yaml                    # SOURCE OF TRUTH; supplied by user
│   ├── constraint-catalog-v2.1.md           # GENERATED ONLY
│   ├── schema.json                          # schema for constraints.yaml
│   ├── README.md
│   └── mappings/
│       └── revision-2-named-constraints.yaml
│
├── docs/
│   ├── spec/
│   │   ├── 00-system-overview.md
│   │   ├── 01-architecture.md
│   │   ├── constraint-catalog.md            # points to generated catalog
│   │   ├── threat-model.md
│   │   ├── residual-risks.md
│   │   ├── isolation-matrix.md
│   │   ├── security-boundaries.md
│   │   └── phases/
│   │       ├── phase-0.md
│   │       ├── phase-1.md
│   │       ├── phase-2.md
│   │       ├── phase-3.md
│   │       ├── phase-4.md
│   │       ├── phase-5.md
│   │       ├── phase-6.md
│   │       ├── phase-7.md
│   │       ├── phase-8.md
│   │       ├── phase-9.md
│   │       └── phase-10.md
│   │
│   ├── adr/
│   │   ├── README.md
│   │   ├── ADR-001-provisional-local-model.md
│   │   ├── ADR-002-pf-privileged-helper.md
│   │   ├── ADR-003-quic-block.md
│   │   ├── ADR-004-profile-lifecycle.md
│   │   ├── ADR-005-tiered-authorization.md
│   │   ├── ADR-006-proxy-scope.md
│   │   ├── ADR-007-headful-background-decision.md
│   │   └── ADR-008-flow-commitment-experiment.md
│   │
│   ├── benchmark/
│   │   ├── pre-registration.md
│   │   ├── gates.md
│   │   └── reports/
│   │
│   ├── security/
│   │   ├── confirmation-dialog-hardening.md
│   │   ├── secret-handling.md
│   │   ├── prompt-injection.md
│   │   ├── network-egress.md
│   │   └── model-supply-chain.md
│   │
│   └── release/
│       ├── signing.md
│       ├── supply-chain.md
│       ├── migrations.md
│       └── residual-risk-acceptance.md
│
├── models/
│   ├── README.md
│   ├── manifests/
│   │   ├── phase1-provisional.json
│   │   └── production.json
│   ├── provisional/
│   │   └── qwen3-1.7b/
│   │       └── README.md                    # binary supplied through LFS/release artifact
│   ├── production/
│   │   └── <model-id>/
│   │       └── README.md
│   └── canary/
│       └── behavioral-tests.json
│
├── src/                                    # React + TypeScript thin Tauri UI
│   ├── main.tsx
│   ├── App.tsx
│   ├── ipc/
│   │   ├── client.ts
│   │   ├── schemas.ts
│   │   ├── allowlist.ts
│   │   └── errors.ts
│   ├── state/
│   │   ├── app.ts
│   │   ├── task.ts
│   │   ├── settings.ts
│   │   └── selectors.ts
│   ├── screens/
│   │   ├── onboarding/
│   │   ├── task-input/
│   │   ├── task-monitor/
│   │   ├── parked-tasks/
│   │   ├── settings/
│   │   └── security/
│   ├── components/
│   │   ├── preview/
│   │   │   ├── ScreenshotPreview.tsx
│   │   │   ├── CanonicalStatePreview.tsx
│   │   │   └── PreviewGuard.tsx
│   │   ├── confirm/
│   │   │   ├── ConfirmationShell.tsx
│   │   │   ├── ConfirmationSummary.tsx
│   │   │   ├── HighStakesPrompt.tsx
│   │   │   └── HardenedText.tsx
│   │   ├── tasks/
│   │   ├── status/
│   │   └── common/
│   ├── tauri/
│   │   └── commands.ts
│   └── styles/
│
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   ├── default.json
│   │   └── restrictive.json
│   ├── icons/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── ipc.rs
│   │   ├── error.rs
│   │   ├── config.rs
│   │   │
│   │   ├── core/
│   │   │   ├── orchestrator/
│   │   │   │   ├── task_normalizer.rs
│   │   │   │   ├── permission_check.rs
│   │   │   │   ├── task_authority.rs
│   │   │   │   ├── progress.rs
│   │   │   │   ├── state_machine.rs
│   │   │   │   ├── resume.rs
│   │   │   │   ├── human_checkpoint.rs
│   │   │   │   ├── budgets.rs
│   │   │   │   └── intervention_epoch.rs
│   │   │   │
│   │   │   ├── perception/
│   │   │   │   ├── pipeline.rs
│   │   │   │   ├── mutation_watcher.rs
│   │   │   │   ├── noise.rs
│   │   │   │   ├── ax_extract.rs
│   │   │   │   ├── dom_extract.rs
│   │   │   │   ├── shadow_dom.rs
│   │   │   │   ├── frames.rs
│   │   │   │   ├── actionability.rs
│   │   │   │   ├── geometry.rs
│   │   │   │   ├── unicode.rs
│   │   │   │   ├── deception.rs
│   │   │   │   ├── compress.rs
│   │   │   │   ├── graph.rs
│   │   │   │   ├── reference.rs
│   │   │   │   ├── stabilize.rs
│   │   │   │   ├── livelock.rs
│   │   │   │   ├── epoch.rs
│   │   │   │   └── fallback.rs
│   │   │   │
│   │   │   ├── reasoning/
│   │   │   │   ├── session.rs
│   │   │   │   ├── schema.rs
│   │   │   │   ├── grammar.rs
│   │   │   │   ├── triggers.rs
│   │   │   │   ├── timeout.rs
│   │   │   │   ├── abort.rs
│   │   │   │   ├── canary.rs
│   │   │   │   └── uncertainty.rs
│   │   │   │
│   │   │   ├── policy/
│   │   │   │   ├── engine.rs
│   │   │   │   ├── classes.rs
│   │   │   │   ├── tiers.rs
│   │   │   │   ├── scopes.rs
│   │   │   │   ├── taint.rs
│   │   │   │   ├── provenance.rs
│   │   │   │   ├── declassify.rs
│   │   │   │   ├── reconcile.rs
│   │   │   │   ├── locale_amount.rs
│   │   │   │   ├── recovery_mode.rs
│   │   │   │   ├── rate_limit.rs
│   │   │   │   └── confirmation.rs
│   │   │   │
│   │   │   ├── execution/
│   │   │   │   ├── primitives.rs
│   │   │   │   ├── hittest.rs
│   │   │   │   ├── typing.rs
│   │   │   │   ├── secure_fill.rs
│   │   │   │   ├── serialization.rs
│   │   │   │   ├── idempotency.rs
│   │   │   │   └── retry.rs
│   │   │   │
│   │   │   ├── verification/
│   │   │   │   ├── outcomes.rs
│   │   │   │   ├── evidence.rs
│   │   │   │   ├── tiers.rs
│   │   │   │   ├── postconditions.rs
│   │   │   │   ├── network_evidence.rs
│   │   │   │   └── download_check.rs
│   │   │   │
│   │   │   ├── skills/
│   │   │   │   ├── store.rs
│   │   │   │   ├── versions.rs
│   │   │   │   ├── commitment.rs
│   │   │   │   ├── pipeline.rs
│   │   │   │   ├── sanitize_allowlist.rs
│   │   │   │   ├── shadow.rs
│   │   │   │   ├── promotion.rs
│   │   │   │   ├── revocation.rs
│   │   │   │   └── drift.rs
│   │   │   │
│   │   │   └── vault/
│   │   │       ├── keys.rs
│   │   │       ├── items.rs
│   │   │       ├── import.rs
│   │   │       └── auth.rs
│   │   │
│   │   ├── cdp/
│   │   │   ├── connection.rs
│   │   │   ├── raw.rs
│   │   │   ├── targets.rs
│   │   │   ├── sessions.rs
│   │   │   ├── frames.rs
│   │   │   ├── contexts.rs
│   │   │   ├── loaders.rs
│   │   │   └── lifecycle.rs
│   │   │
│   │   ├── browser/
│   │   │   ├── process.rs
│   │   │   ├── profiles.rs
│   │   │   ├── highrisk_queue.rs
│   │   │   ├── downloads.rs
│   │   │   ├── permissions.rs
│   │   │   └── watchdog.rs
│   │   │
│   │   ├── net/
│   │   │   ├── proxy.rs
│   │   │   ├── canonical.rs
│   │   │   ├── intercept.rs
│   │   │   ├── classify.rs
│   │   │   ├── tls.rs
│   │   │   └── pf.rs
│   │   │
│   │   ├── inference/
│   │   │   ├── llama.rs
│   │   │   ├── pin.rs
│   │   │   ├── memory.rs
│   │   │   └── install.rs
│   │   │
│   │   ├── audit/
│   │   │   ├── chain.rs
│   │   │   ├── encrypt.rs
│   │   │   ├── access.rs
│   │   │   └── redact.rs
│   │   │
│   │   ├── storage/
│   │   │   ├── sqlite.rs
│   │   │   ├── migrations/
│   │   │   └── models/
│   │   │
│   │   ├── security/
│   │   │   ├── unicode_tables.rs
│   │   │   ├── idn.rs
│   │   │   ├── trust_store.rs
│   │   │   └── clocks.rs
│   │   │
│   │   └── app/
│   │       ├── lifecycle.rs
│   │       ├── settings.rs
│   │       └── diagnostics.rs
│   │
│   ├── fuzz/
│   │   └── fuzz_targets/
│   │       ├── cdp_parser.rs
│   │       ├── canonical_request.rs
│   │       ├── unicode_norm.rs
│   │       ├── schema_validate.rs
│   │       ├── skill_commitment.rs
│   │       └── tier_derivation.rs
│   │
│   └── tests/
│       ├── lifecycle/
│       ├── browser/
│       ├── network/
│       ├── egress/
│       ├── isolation/
│       ├── policy/
│       ├── verification/
│       ├── resume/
│       └── security/
│
├── helper/
│   ├── README.md
│   ├── pf-helper/
│   │   ├── Cargo.toml
│   │   └── src/
│   └── install/
│
├── mock-sites/
│   ├── server.ts
│   ├── sites/
│   │   ├── navigation/
│   │   ├── forms/
│   │   ├── shopping/
│   │   ├── payment/
│   │   ├── dynamic/
│   │   ├── oopf/
│   │   └── deception/
│   └── injectors/
│       ├── reordering.ts
│       ├── aria-mutation.ts
│       ├── layout-shift.ts
│       ├── target-replacement.ts
│       ├── overlay-clickjacking.ts
│       ├── prompt-injection.ts
│       ├── fake-success.ts
│       ├── malicious-redirect.ts
│       ├── url-exfiltration.ts
│       ├── credential-harvesting.ts
│       ├── deceptive-payment.ts
│       ├── element-swap.ts
│       ├── tier-downgrade.ts
│       ├── threshold-splitting.ts
│       ├── unicode-homograph.ts
│       └── ...
│
├── bench/
│   ├── tasks/
│   │   ├── dev/
│   │   ├── heldout/
│   │   └── external/
│   ├── harness/
│   │   ├── runner/
│   │   ├── thermal-soak/
│   │   └── realistic-workload/
│   ├── corpus/
│   │   └── classification/
│   │       ├── actions.jsonl
│   │       ├── labels.jsonl
│   │       └── README.md
│   ├── traces/
│   └── analysis/
│
├── tools/
│   ├── audit-verify/
│   ├── model-pin/
│   ├── catalog-lint/
│   ├── catalog-generate/
│   └── release/
│
├── scripts/
│   ├── bootstrap.sh
│   ├── verify-model.sh
│   ├── generate-catalog.sh
│   ├── lint-catalog.sh
│   ├── run-phase-tests.sh
│   └── release-check.sh
│
└── .github/
    └── workflows/
        ├── ci.yml
        ├── security.yml
        ├── fuzz.yml
        └── release.yml
```

## 4. Core architectural rules to implement

### Orchestration

The system has five engines coordinated by the orchestrator/task-normalizer/permission-check layer. Maintain an orchestrator-owned typed progress object and deterministic state transitions.

### Action surface

The model may request only typed operations:

- NAVIGATE
- CLICK
- TYPE
- SELECT
- SCROLL
- WAIT
- PRESS_KEY
- SECURE_FILL
- REQUEST_CONFIRMATION

Do not implement arbitrary model-controlled JavaScript execution.

Bind model-proposed actions to the state epoch from which they were derived. Reject stale actions after relevant state mutation and force fresh perception/reasoning.

### Policy

Keep side-effect classes distinct from authorization tiers:

- READ
- REVERSIBLE_WRITE
- IRREVERSIBLE
- UNKNOWN -> IRREVERSIBLE

Authorization tier is derived by trusted Policy, never by the model. Keep the tier fail-upward and bind authorization commitments to the approved tier.

Biometric confirmation is reserved for HIGH_STAKES actions such as payments/financial actions and important sensitive decisions. Do not add biometrics to ordinary navigation, typing, credential filling, reversible actions, or routine execution.

The model must only emit `REQUEST_CONFIRMATION`; it must not know whether the request will become ordinary native confirmation or a biometric prompt.

### Security / untrusted input

Treat all page-derived content and model prose reflecting page-derived content as untrusted. No page-controlled content may redefine task authority, policy, authorization tier, credential scope, or trusted confirmation data.

Never expose raw secrets, cookies, authentication headers, or secret values to model context or verification evidence.

### Verification

Verification is independent of model self-report. Maintain the five outcome classes and independent evidence sources. Page banners and model self-report are not sufficient evidence of success.

### Browser runtime

Use managed Chromium over CDP, preferably through `chromiumoxide`, with raw CDP available where necessary. Use `--remote-debugging-pipe`, not a TCP debugging endpoint. Preserve sandbox and site isolation. No forbidden convenience flags.

Implement target/session/frame/loader lifecycle tracking, OOPIF handling, downloads, permissions, watchdogs, memory-aware process management, and safe shutdown/resumption.

### Network

Proxy authority is destination/egress control. Do not implement trusted HTTPS body interception as part of the proxy design.

Implement request correlation, default-deny on correlation miss, proxy-death fail-closed behavior, QUIC/UDP 443 blocking strategy, DoH-disabled behavior, and the privileged `pf` helper boundary.

### Profiles

High-risk browser profiles are ephemeral by default. Persistent per-site profiles require explicit opt-in, expiry, storage caps, and purge.

### Local model

The model is a repository-controlled build/release artifact. Manifest + SHA256 remain in normal Git; multi-GB GGUF binaries live in Git LFS or signed release artifacts.

The application must load the model only from:

`~/Library/Application Support/<app>/models/`

and verify the hash against the manifest on every load.

The application must never download models at runtime from Hugging Face, OpenAI, Anthropic, or another external model service.

### Inference

Implement llama.cpp/Metal integration, pinned model metadata, structured output/grammar support, timeout/abort wiring, memory-pressure handling, behavioral canaries, and uncertainty-triggered reasoning.

The Phase-1 provisional model is the catalog/spec-selected candidate. Do not silently substitute another model. Production promotion happens only after the catalog-defined benchmark gate.

### Skills

Skills are immutable versioned artifacts with semantic commitments, origin binding, required roles/fields, invariants, policy requirements, drift detection, revocation, shadow evaluation, and allowlist-defined memory.

Never store secrets, credential values, token-bearing URLs, or arbitrary page content in skill memory.

### Vault

Implement the catalog-defined key hierarchy and password-change key re-wrap behavior without breaking audit-chain verification.

### Audit

Implement encrypted append-only HMAC-chained audit segments, rotated-segment linkage, redaction, retention, export, and local verification.

### Resource budgets

Create named budget controls for all catalog-defined hard limits, even where numeric values remain undefined. A missing numeric value must not silently become a permanent security assumption; surface it as a tunable configuration blocked from being declared final until the catalog resolves it.

## 5. Catalog enforcement tooling

Generate a catalog tooling package under `tools/catalog-lint/` and scripts under `scripts/`.

It must enforce at minimum:

1. IDs unique.
2. IDs stable and sequentially present; no existing IDs deleted or renumbered without explicit SUPERSEDED metadata.
3. Spec status is one of: LOCKED, TUNABLE, PROVISIONAL, MEASUREMENT-ADR, PROPOSED, SUPERSEDED.
4. Verification status is one of: NOT-VERIFIABLE-YET, SCHEDULED, PARTIAL, VERIFIED, FAILED-BLOCKING.
5. Every LOCKED constraint has an owner phase and named test ID.
6. Named Revision-2 constraints resolve to catalog IDs.
7. Every `depends_on`, `amends`, and `supersedes` reference resolves.
8. `value_defined: false` entries are surfaced.
9. If an undefined-value constraint's owner phase has started, CI raises a warning or configured blocking condition rather than inventing a value.
10. PROPOSED entries cannot be used as release-blocking gates by Phase 0–7.
11. PRs touching `src-tauri/src/core/policy/**` must reference affected C-IDs.
12. Generated Markdown must exactly match YAML content after generation.
13. Catalog schema validates before build/test pipelines continue.

## 6. Test architecture

Every major subsystem needs:

- unit tests
- deterministic fixture tests
- negative/security tests
- integration tests where cross-component behavior matters
- fault-injection tests for fail-closed behavior
- explicit C-ID references in test metadata

Create a registry that maps:

`C-ID -> test_id(s) -> test file(s) -> phase -> evidence artifact`

Do not mark verification status automatically from static analysis alone.

## 7. Phase implementation order

Implement with the following ownership boundaries:

### Phase 0
Freeze catalog integration, ADRs, threat model, isolation matrix, provisional model manifest, repo/build conventions, and baseline CI.

### Phase 1
Build the deterministic vertical slice: orchestrator, perception slice, reasoning slice, policy slice, typed execution, independent verification, mock sites, safe hard-stop/resume semantics, progress object, epoch binding, budgets, audit foundation, provisional model installation path.

### Phase 2
Harden Chromium, sessions/targets/frames, profile isolation, network/proxy/pf boundary, QUIC/DoH behavior, downloads, watchdogs, lifecycle, resource caps, and safe termination.

### Phase 3
Complete perception, DOM/AX/OOPIF handling, geometry/actionability, Unicode/deception defenses, semantic compression, fallback ladder, recursive OOPIF hit testing, hybrid typing, stabilization/livelock.

### Phase 4
Complete Policy, action classes, authorization tiers, taint/provenance, credential scope, secure fill, task-authority reconciliation, confirmation policy, rate limits, vault integration.

### Phase 5
Complete independent verification, evidence hierarchy, confirmation-dialog hardening, approval commitments/tier binding, audit chain, outcome handling, root-cause taxonomy, latency gates.

### Phase 6
Complete local inference, model selection benchmark, canary, timeout/abort, memory pressure handling, production promotion workflow, uncertainty-triggered reasoning, and metric separation.

### Phase 7
Complete skills/macros/immutable versions/commitments, shadow replay, drift detection, revocation, promotion pipeline, generalized benchmark traces.

### Phase 8
Implement the proposed integration/recovery expansion behind explicit feature/phase gates: long controlled workflows, human checkpoints, intervention epochs, parked tasks, manual takeover authority boundaries, crash-safe explicit resumption, notification expiry, and UX measurements. Do not treat Phase 8 as proof of broad real-world autonomy.

### Phase 9
Implement the release-blocking adversarial/security gate and complete the ≥600-action classification corpus, external task set, full injector suite, ablations, red-team suite, residual-risk acceptance, and formal `SECURITY_GATE = PASS | FAIL | CONDITIONAL` output.

### Phase 10
Operational release engineering only: signing, notarization, updater, release-key ceremony, model repin automation, Chromium update process, migrations, fuzzing CI, dependency/SBOM checks, privacy documentation, uninstall/purge, traceability matrix, and reproducible release checks.

## 8. Hard-stop implementation rule

When a safety/security invariant cannot be proved or a required parameter is undefined:

- do not guess a value;
- do not bypass the check;
- do not silently downgrade security;
- stop the affected execution path;
- record the exact C-ID and reason;
- expose `NOT_VERIFIABLE_YET`, `UNKNOWN`, `SAFE_ABORT`, or the catalog-appropriate state;
- leave a deterministic resumption path after the constraint is resolved.

## 9. Definition of done for the IDE

The IDE task is complete only when:

1. The full directory tree exists.
2. All Rust modules compile with explicit module boundaries.
3. Frontend IPC is allowlisted and typed.
4. Catalog YAML validates.
5. Catalog Markdown generation is deterministic.
6. Catalog CI checks execute.
7. Phase ownership is explicit.
8. Every security-critical subsystem has tests and C-ID references.
9. Model manifests, artifact verification, and runtime model-path logic exist.
10. No runtime model download path exists.
11. No arbitrary model-controlled JavaScript path exists.
12. No raw-secret path enters model context.
13. Authorization tier is derived only by Policy.
14. HIGH_STAKES biometric flow is isolated from ordinary execution.
15. Verification is independent of model self-report.
16. Fail-closed behavior exists for proxy death and stale authorization/epoch state.
17. Undefined catalog parameters are surfaced rather than invented.
18. Phase 8–10 proposed scope is gated and never represented as already proven.
19. No claim of VERIFIED is made without executed evidence.
20. The final IDE report lists every unfinished item by C-ID, phase, reason, and next verification action.

## 10. Required final IDE report

At the end of every generation/build pass, emit:

- repository tree created;
- files created/modified;
- compilation status;
- test status;
- C-ID coverage count;
- LOCKED constraints with no test mapping;
- constraints whose verification is still NOT-VERIFIABLE-YET;
- undefined parameters still unresolved;
- PROPOSED constraints implemented behind gates;
- security blockers;
- known stubs or intentional not-yet-implemented interfaces;
- exact next phase / next executable verification step.

Never report "fully implemented" based only on file presence. Report implementation and verification separately.
