# Browser-Agent — Master Product, Design, UX, Architecture and Quality Specification

**Document status:** Authoritative consolidated specification with decisions locked for implementation  
**Revision date:** 2026-10-13 (open-decision closure, operability budgets, privacy/data lifecycle and cross-section consistency pass)  
**Scope:** Full Browser-Agent product, not an MVP-only proposal  
**Target platforms:** macOS on Apple silicon and Windows x64 desktop  
**Application stack:** Rust core, React/TypeScript UI, Chromium Embedded Framework (CEF) rendering host, narrow audited C++/Rust bridge  
**Browser runtime:** Product-managed embedded Chromium through CEF native APIs; CDP is restricted to isolated development/test builds  
**Inference posture:** Local model execution; no required cloud inference, remote model endpoint, or vendor API key  
**Primary product statement:** **A browser that can act for you.**

This document consolidates the prior Browser-Agent product thesis, `DESIGN.md`, the design-system thesis, token guidance, anti-AI-slop rules, Apple Human Interface Guidelines research, browser-runtime requirements and security/verification requirements. It supersedes contradictory earlier design drafts. This document is a product and implementation contract; it is not proof that any feature has already been implemented or passed validation.

---

## 0. Authority, scope and decision precedence

### 0.1 Source-of-truth order

When documents, code, components or coding-agent suggestions conflict, apply this order:

1. **Security, privacy and runtime invariants in this specification.** No visual requirement can weaken them.
2. **The explicit product and UX decisions in this specification**, including the permanent visible Agent panel and restricted icon whitelist.
3. **The canonical seven-color palette and approved semantic token mappings** in Section 6.
4. **The Apple-informed craft and platform guidance** in Section 5 and the official references in Appendix A. Apply relevant principles without copying unrelated Apple visual treatments.
5. **The repository's existing, verified implementation and architecture**, provided it does not violate this specification. Do not assume a documented component exists simply because it is described here.
6. **Library defaults, coding-agent preferences and old design drafts.** These have no authority when they conflict with this document.

### 0.2 Explicit decisions that supersede earlier drafts

- The AI Agent panel is **mandatory and permanently visible** in the application. It is not a feature toggle, optional sidebar, dismissible drawer or hidden-by-default tool.
- The interface is serif-first. Do not use a sans-serif family for first-party Browser-Agent UI. Choose and bundle a suitable screen-readable serif family after inspecting available licensed font assets.
- App-owned icons are restricted to **Undo, Redo, Reload, Bookmarks, Downloads and the three-dot Settings/menu control**. Other first-party controls use clear text labels. Website favicons and icons rendered inside remote websites are not Browser-Agent iconography and may remain.
- Do not render macOS Command/Option/Shift/Control key glyphs. Spell shortcut names out with words.
- Use selective, restrained corner radii where they improve grouping or affordance. Do not round every surface and do not use pill-shaped controls.
- Do not use glow, neon edges, glassmorphism, gradients, ambient shadows, decorative motion or animations added to make the product seem more “AI-like.”
- Enable appropriate lazy loading for non-critical app-owned assets and secondary content. Never lazy-load the browser shell, address bar, persistent Agent panel shell, first interaction targets or other content required for immediate use.
- The actual managed Chromium page must be visible and receive the input/action directed at it. A screenshot, placeholder, simulated page or unrelated WebView is not an acceptable substitute.
- Normal manual browsing must work without a local model, provided the required browser and security runtime prerequisites are available.
- Autonomous payment submission is out of scope. The agent may prepare work and hand control to the user for payment or another consequential action.
- Do not use fake data, fake progress, simulated task success or decorative technical telemetry.
- Do not use “v1”/“v2” as vague marketing labels or as a substitute for a meaningful specification decision. Accurate build, app, model, CEF and diagnostic version identifiers must remain available in About/Diagnostics and release metadata where they identify actual artifacts.

### 0.3 Required frontend craft process

[BA-DESIGN-001] If the development environment contains the locally installed Impeccable design skill, the frontend implementation agent must read and use it for relevant frontend design and quality work. The skill is a development aid, not product code; do not vendor it into the application. Its recommendations must not override this specification's explicit decisions.

If Impeccable is absent or unavailable, do not block implementation and do not install a substitute automatically. Apply Sections 3–8 and 18 directly, use the official references in Appendix A, inspect rendered screenshots for all required states in Section 21.1, and record findings in the same design-review checklist. The quality process is mandatory regardless of tool availability; the skill is optional assistance, not the quality gate.

---


### 0.4 Binding architecture decisions

The decisions below resolve architecture ambiguities in this specification. They are binding unless a later, evidence-backed architecture decision record supersedes them. The implementation agent must not ask the user to make these technical choices again.

#### ADR-010 — Use CEF as the single Chromium rendering host; do not ship a Tauri/Wry plus external-Chromium composition

**Decision:** Retain Rust as the trusted application core and React/TypeScript as the UI implementation language, but use CEF (Chromium Embedded Framework) as the native window's browser/rendering host. Remove Tauri/Wry from the production rendering path and do not launch a separate stock Chromium window that must be visually reparented into the app. Use CEF Views/BrowserView-style native composition so app-owned chrome and the real remote webpage are views in one native window and one supported browser engine.

**Why:** Tauri's documented model renders its UI through the operating system WebView, while the original design also required a separate managed Chromium process to occupy part of the same window. CEF is explicitly built for embedding browser controls into native applications. Keeping two independent browser rendering systems and trying to overlay/reparent a foreign process creates avoidable input, focus, z-order, resize, accessibility, lifecycle, and cross-platform risks. CEF's native view model is the more direct fit for a browser product.

**Implementation boundary:** CEF's C++ API is wrapped by a small, audited C ABI adapter; Rust owns task orchestration, policy, authorization, credentials, audit, resource governance and persistence. React/TypeScript supplies the app-owned UI, loaded only from the fixed registered first-party origin `browser-agent://ui/`. CEF owns the native window/view layout and real webpage rendering. All CEF calls are marshaled to the required CEF thread. The C++ adapter may not implement policy decisions or hold credentials.

**Context separation:** Use separate CEF request contexts/storage partitions for first-party app UI, normal browsing and Private Windows. The app UI uses `browser-agent://ui/` and no persistent website cookies; all normal windows/tabs share a dedicated app-owned browsing profile by default so normal website sign-in sessions work across tabs. Each Private Window uses its own ephemeral context as defined in Section 2.16 and never shares cookies/site state with the normal profile or another Private Window. Register the privileged native message bridge only for the exact `browser-agent://ui/` origin and approved app UI frames. Remote webpage frames receive no privileged bridge, credential references, filesystem API, shell API or generic native command capability. Remote tabs may not navigate into the privileged app scheme.

**CDP decision:** Do not expose a production CDP endpoint. Production browser control uses the narrow CEF/native bridge and renderer observation protocol, with the policy layer independently authorizing every action. CDP may be enabled only in an explicit development/test build, on a disposable test profile, and must be disabled in release artifacts. Do not ship `chromiumoxide` in the production runtime merely to connect to a local debugging port.

**Native surface proof remains mandatory [BA-ARCH-002]:** Although CEF is the selected architecture, it is not considered proven until the feasibility gate passes on macOS Apple silicon and Windows x64. The gate validates real multi-view composition, actual page rendering, pointer/keyboard/IME delivery, native resize/focus, separate storage contexts, multiple-window lifecycle, crash recovery, accessibility and clean packaging. The gate validates the chosen architecture; it is not a request for the user to choose among alternatives.

**Owner and timebox:** The CEF/runtime architecture owner is the directly responsible engineer recorded in `CODEOWNERS` and the ADR. The repository maintainer is accountable for closing the gate. The gate has a maximum of **10 business days** from the first reproducible build: days 1–5 for the windowed native-view proof and days 6–10 for root-cause remediation and, only if needed, a CEF off-screen-rendering feasibility proof. Progress is recorded daily as test evidence, not status prose.

**Failure path:** If neither supported CEF rendering approach passes every mandatory criterion by the end of day 10, mark ADR-010 `NO-GO`, block downstream UI implementation/release work that depends on the browser surface, and publish a report containing the failing platform/API, reproduction, logs, and the cheapest technically credible remediation. The architecture owner must record the final engineering disposition within a further **5 business days**. No one may silently return to Tauri plus a floating Chromium window, replace the browser with a screenshot, or downgrade to a platform WebView. Work not dependent on the browser surface may continue; dependent implementation remains blocked until a passing proof and explicit ADR revision exist. This is a finite stop condition, not permission to wait indefinitely or ask the end user to debug the integration.

#### ADR-011 — Standardize on llama.cpp for local inference

Use one local inference engine across supported platforms: llama.cpp, selected and pinned after benchmarks. Use the Apple Metal backend on Apple silicon when the tested build supports it; on Windows use the validated CPU path as the baseline and enable a GPU backend only when it passes the same safety and performance gates on the target hardware. Do not add MLX as a second production runtime. The model runner communicates with Rust through managed stdin/stdout IPC or a similarly private OS process channel, never an unauthenticated localhost HTTP server. Rust validates every response against the typed action schema.

The model and model files are optional for manual browsing. Missing, failed, parked, incompatible or resource-stopped inference must not disable ordinary browsing. No cloud inference fallback is permitted.

#### ADR-012 — Local start page; no unsolicited first-launch request

First launch opens a first-party local new-tab page, not an external search provider. The address bar remains ready for direct navigation or search. A default search provider is configurable; use DuckDuckGo as the initial default for the privacy-conscious target audience, disclose that search requests go to that provider, and allow the user to switch providers. Do not claim that a selected search provider guarantees privacy. No external page is requested until the user navigates or submits a search.

#### ADR-013 — Separate browser baseline from agent capability baseline

Manual browser capability is the baseline product requirement. Agent capability is an optional local subsystem that becomes available after a compatible model is installed and passes readiness checks. A model/runtime failure degrades the Agent, not browsing. A policy, audit, profile-isolation or browser-integrity failure blocks the affected sensitive operation and must never trigger a remote or less-secure fallback.

### 0.5 Glossary

Use these definitions consistently throughout the specification, schemas, code and diagnostics.

| Term | Normative definition |
|---|---|
| **Application window** | One native top-level Browser-Agent window, with its own tabs and persistent Agent panel. |
| **Browser profile** | The persistent local browsing-data boundary used by normal windows, including cookies, site storage, history and related browser state. |
| **Private window** | A window using a separate ephemeral CEF request context that does not persist browsing data to the normal profile. It is not a guarantee of anonymity from websites, networks or the operating system. |
| **Task** | One user-requested workflow with one Rust-owned lifecycle state machine and a terminal outcome. |
| **Observation epoch** | Monotonically increasing identity for one coherent page observation. Material navigation, target/frame replacement, takeover or relevant semantic mutation invalidates dependent target references and authorizations. |
| **Semantic state graph** | A bounded, structured representation of current actionable elements, accessible semantics, relationship context and frame/origin identity, scoped to an observation epoch. It is untrusted page evidence, not policy authority. |
| **Stabilization window** | A bounded interval in which the runtime evaluates whether the page has reached a sufficiently stable state for the current action-specific precondition. It always has a deadline and cancellation path. |
| **Authorization** | A Rust-policy decision bound to one action, parameters, target, origin, frame, epoch, task and single-use expiry/consumption state. It is not user confirmation text. |
| **Compensating operation** | A separately authorized action with an evidenced inverse/postcondition that attempts to restore a proven prior state. It does not erase external side effects unless that is independently established. |
| **Verification outcome** | Evidence assessment of the requested postcondition, separate from whether the task lifecycle terminated or failed. |
| **Telemetry** | Data sent off-device by Browser-Agent about app operation. Website requests explicitly caused by browsing and user-requested downloads are not Browser-Agent telemetry. |
| **Local diagnostic export** | An export explicitly initiated by the user, previewed/redacted before writing or sharing, and not uploaded automatically. |
| **Safe checkpoint** | A durable, redacted record of the last reconciled browser/task state from which automation can resume only after identity validation, fresh perception and reauthorization. A checkpoint is not proof that an in-flight external side effect did or did not occur. |
| **Reconciliation** | Comparing the current live browser/runtime state against a saved checkpoint and task postconditions, identifying drift and ambiguous side effects before any action resumes. |
| **Task-owned tab set** | The originating tab plus tabs explicitly created/approved within the owning task's window. The Agent may act only in this set unless a user explicitly grants another current tab. |
| **Egress** | Any Agent-caused disclosure of task/page/user-derived data to a remote origin, including typed field values, URL parameters, form submissions, downloads requests or external-protocol launches. |
| **Parked inference** | A model runner that has been stopped/unloaded to release resources. No model inference is in progress; manual browsing remains available. |
| **Skill** | An immutable, versioned task procedure with tested preconditions, target evidence, allowed action types, compensation rules, postconditions and provenance. A skill never bypasses fresh perception or policy. |
| **Task budget** | Independent ceilings on task active duration, authorized action attempts, Agent-initiated top-level navigations, and cumulative inference tokens. A budget limit is separate from lifecycle state and verification outcome. |
| **Requirement registry** | The machine-readable source of stable requirement IDs, ownership, test IDs, evidence expectations, priorities and status; Appendix D is generated from it. |

### 0.6 Requirement IDs and traceability contract

[BA-TRACE-001] Every release-critical requirement has a stable ID embedded next to its normative text and repeated in the requirement-to-evidence matrix. Use the namespace `BA-<AREA>-NNN`, where `AREA` is a stable subject such as `ARCH`, `BROWSER`, `POLICY`, `PRIVACY`, `MODEL`, `UI`, `A11Y`, `BUILD` or `RELEASE`. IDs are immutable after assignment: moving a paragraph does not change its ID, retired IDs are never reused, and new requirements receive the next unused ID in that area.

The matrix must point to the exact ID, implementation owner/module, test identifier, evidence artifact, status and any approved deviation. It must not invent IDs locally or infer that a requirement passed because code exists. If one paragraph contains requirements with different tests or owners, split it into separate IDs. The canonical machine-readable ID inventory is `requirements/registry.yaml`; Appendix D is generated from it. The inline ID remains authoritative for interpretation of the requirement.

### 0.7 How this specification is amended

[BA-AMEND-001] This specification is normative. An amendment must be submitted as a reviewed change that identifies the affected requirement IDs, current behavior, proposed behavior, rationale, security/privacy/accessibility impact, tests/evidence, dependent sections, and any ADR that changes. A change that affects architecture, security boundaries, egress, supported browser capabilities, privacy promises, model runtime or release gates requires an ADR and updates to the threat model before implementation begins. The same change must update `requirements/registry.yaml`, Appendix D's generated inventory, the requirement-to-evidence matrix, relevant schemas and tests. No implementation may silently diverge from the specification.

[BA-REQ-LINT-001] `requirements/registry.yaml` is the machine-readable source of truth for stable requirement IDs. `docs/DESIGN-FINAL.md` contains the human-readable generated registry in Appendix D. A repository script validates unique IDs, inline-to-registry coverage, valid source sections, test IDs for every release-critical requirement, known status values, and absence of orphan/unknown IDs. CI fails if generated Appendix D differs from the source registry. A requirement-density check flags normative paragraphs containing multiple independent obligations with different owners/tests for human review; this check is advisory for prose but may not be used to omit a requirement from the registry. Release-critical requirement coverage must be 100% before the release gate.

[BA-THREAT-001] Maintain `docs/THREAT_MODEL.md` as a first-class, version-controlled artifact. It must cover trust boundaries, assets, actors, attacker capabilities, entry points, prompt injection, page-to-agent egress, renderer/native bridge, CEF contexts, private mode, credential vault, model runner IPC, audit integrity, downloads, local services and update/install supply chain. Every mitigation maps to one or more requirement IDs and negative tests. Review the threat model for changes to CEF, Rust bridge, policy, vault, downloads, model runner, network behavior or packaging.

[BA-REVIEW-001] Every release candidate receives a recorded design/security review by the accountable release owner named through `CODEOWNERS`. If the project is maintained by one person, the maintainer remains accountable but must rely on executed automated evidence and must not treat self-review prose as a substitute for the mandatory cross-platform, accessibility, adversarial and release tests. The review must include Section 21.1 screenshots in both OS-following themes, supported scales, minimum and normal window sizes, and blocked/unknown states.

## 1. Product thesis and user promise

Browser-Agent is a local-first desktop browser with an autonomous control layer. The user browses the web normally, then delegates a multi-step task to an agent that perceives the real webpage, proposes a typed action, passes through a trusted authorization policy, executes through the managed browser and independently verifies the result.

The product is not:

- an AI dashboard with an embedded browser preview;
- a generic chatbot with a browser tool;
- a cloud automation service requiring a remote model API;
- a macro recorder that depends entirely on fixed selectors;
- a system that lets the model authorize its own actions;
- an autonomous-payment product;
- a guarantee that every dynamic website or workflow will always succeed.

The defining separation is:

1. **What the user requested.**
2. **What the page currently shows.**
3. **What the model proposes.**
4. **What policy authorizes.**
5. **What the browser actually executes.**
6. **What verification can establish happened.**

These concepts must never be collapsed into a single “AI did it” status.

### 1.1 Core product goals

- Let users delegate real multi-step web workflows in natural language.
- Keep ordinary browsing familiar and available without inference.
- Make agent activity, control ownership, interruption and verification understandable.
- Keep inference and agent decision-making local by default, with no required cloud model or API key.
- Minimize disclosure of browser context, credentials, and sensitive data to the model and frontend.
- Make policy enforcement independent of model output.
- Make the agent useful without requiring the user to learn internal technical concepts.
- Prefer correctness, recovery and honest uncertainty over impressive-looking demonstrations.

### 1.2 Target user hypothesis

Initial target users are privacy-conscious, technically capable desktop users who perform repetitive or multi-step work in websites and value being able to inspect and interrupt automation. This is a target hypothesis, not validated product-market fit. Do not make unsupported claims about market dominance, competitor weakness or adoption.

### 1.3 Representative workflows

- Research across several websites, collecting source-grounded notes.
- Compare products, plans, specifications or options and open a selected result.
- Complete repetitive non-consequential web administration.
- Fill ordinary forms using authorized data and a secure credential pathway.
- Prepare shopping or booking workflows, then hand control to the user before payment or another high-impact submission.
- Reuse validated skills when current-page preconditions match, while still re-perceiving and re-authorizing each action.

### 1.4 User promise

The agent can attempt a task in the real browser, explain its current operational status in plain language, ask when authority is needed, allow immediate human takeover and report only what the available evidence supports.

### 1.5 Intended differentiation and unproven assumptions

The product is designed around these potential differentiators:

- **Browser-native workflow:** the real webpage remains the place where work happens.
- **Grounded perception:** actions refer to current page state, not imagined controls or stale screenshots.
- **Independent authorization:** the Rust policy layer decides what may execute.
- **Evidence-based verification:** the system distinguishes an attempted action from a proven result.
- **Human control:** pausing, takeover, confirmation and resume are real runtime transitions.
- **Local-first processing:** no required cloud inference, with secrets and agent context kept local by default.
- **Reusable skills:** validated workflow components that still re-perceive and re-authorize each action.

These are intended differentiators, not proven market advantages. Validate them with real performance, task-success, recovery, privacy and security measurements. Do not claim that competitors universally lack these features or that a market category is empty without current, source-backed research.

Key unproven assumptions include local-model capability within the hardware budget, robust perception across changing websites, user trust in an observable Agent, meaningful demand for privacy/local control, dependable cross-platform Chromium integration and the usefulness of reusable skills. Keep these assumptions visible in engineering/product planning instead of converting them into marketing claims.

The business model and go-to-market path are not fixed by this specification. Do not add pricing UI, payment/subscription flows, fake enterprise features or growth-oriented dashboard elements to the application without an explicit product decision. The prohibition on autonomous payment/order submission is a product safety boundary, not a claim about the viability of any broader market category.

---

## 2. Product experience and layout contract

### 2.1 Browser-first structure

The application must read as a browser at first glance. Its visual hierarchy is:

1. Real webpage content and browser viewport.
2. Tabs and normal browser navigation/address controls.
3. Persistent Agent panel, always visible but compact and subordinate to the webpage.
4. Required intervention, confirmation or verification details.
5. Secondary browser/application features.
6. Advanced diagnostics and raw technical events.

Do not make a dashboard, event ledger, model console, statistics page, chat transcript or marketing home screen the default surface.

### 2.2 Main layout

Desktop layout is a stable two-region working surface beneath the tab strip and navigation toolbar:

- **Browser region:** a real, interactive managed Chromium surface.
- **Agent region:** a persistent panel with task input, real status and task activity.

The Agent panel must not be hidden by a close button, collapse control, “show Agent” toggle or user preference. Its exact geometry may adapt to the available window size, but its essential controls and live status remain on-screen.

Recommended starting geometry on a sufficiently wide window:

- Agent panel target width: 320 px.
- Agent panel preferred range: 280–360 px.
- Browser region receives all remaining width.
- One-pixel or two-pixel boundaries provide separation; no large shadow is needed.

These are starting targets, not a reason to violate readable minimums. Measure at common laptop widths and high-DPI scales. At narrower widths, change the layout to a stacked split: browser content above and a persistent Agent work area beneath it. The Agent may become more compact, but it must not disappear or become optional. Keep both regions directly available without a modal or hidden tab.

Do not promise that the page always occupies 85% of the window while the Agent remains permanently visible; that is geometrically impossible in many useful window sizes. Instead, preserve the maximum practical webpage area for each supported viewport and record the tested minimum width/height at which each layout changes.

### 2.3 First launch

- Launch directly into the browser workspace, not a landing page or onboarding wizard.
- Load a first-party local new-tab page on first launch. It must not make an external network request merely because the app opened.
- The actual CEF webpage surface, URL, focused target and displayed page must correspond to the same managed CEF browser instance.
- If CEF cannot initialize or a required security prerequisite fails, show a clear actionable error; do not silently substitute a screenshot, system-default browser, unmanaged external browser or platform WebView.
- Do not require sign-in, model download, cloud account or model setup before ordinary browsing can work.
- If no compatible model is installed or loaded, manual browsing remains operational and the Agent area states the precise readiness condition, such as “Model not installed,” with a real setup action only if one exists.
- The default search provider is DuckDuckGo, configurable in Settings. Search-provider requests occur only after the user submits a search or explicitly invokes a provider. Clearly disclose the provider used; do not imply that the choice guarantees anonymity or prevents network-level tracking.
- First-run model installation is opt-in, integrity-checked and cancellable. Never download a model silently as a prerequisite to manual browsing.

### 2.4 Tabs

- Tabs are familiar browser task containers.
- Show the page title and, where available, the website's actual favicon. These site-provided favicons are exempt from the app-owned icon whitelist.
- Make the selected tab unmistakable through surface, border and text hierarchy—not a glowing ring.
- Keep tab height compact but readable; target approximately 32–36 px with scaling support.
- Support tab overflow without making the overall page horizontally scroll.
- Loading, crashed, navigation-pending and loaded states must be distinct and derived from actual browser runtime events.
- A tab may include text or restrained state treatment for real Agent activity, but no invented activity or decorative badge. When CEF reports active audio, the tab presents the textual `Audio playing` status and its text-only context menu offers `Mute tab` or `Unmute tab` according to the real state; no mute icon is added.
- Provide expected tab operations through text-labelled context menus and keyboard shortcuts.

[BA-TABSCOPE-001] Tabs support in-window reordering and tab tear-off into a new application window only when no Agent task is active. While a task is active, dragging a task-owned tab outside its owning window is blocked with a text explanation; this prevents an active task's authority binding from changing underneath it. A page-created `window.open` or `target=_blank` must never create a chromeless/native unframed window. Route eligible new targets to a normal tab in the owning window after origin/popup policy evaluation; unexpected or disallowed targets are blocked or shown as an explicit user-mediated tab opening. A site-created popup never inherits the opener's Agent authorization automatically.

[BA-SESSION-001] Offer local session restoration for normal browsing only. Restore normal tab URLs and the last active normal tab after a crash/restart only when the user has enabled session restore; never restore private windows, private URLs, Agent tasks, pending approvals or in-flight action state. Restored tabs are new page loads and must not inherit stale node IDs, authorization tokens or observation epochs. Session restore does not restore secret form values or replay form submissions.

[BA-ZOOM-001] Remember user-selected page zoom locally per normal-profile origin and restore it for that origin. Private-window zoom state is ephemeral and destroyed when that window closes. Zoom changes invalidate element geometry and observation epochs before another Agent action. Provide a text-labelled `Reset zoom` menu command; do not add another app-owned icon.

### 2.5 Navigation and address controls

- Provide Back and Forward as **text-labelled controls**, not custom arrow icons.
- Provide Reload as a permitted icon control. It may present Stop while a real navigation is in progress, but do not invent a second unrelated icon.
- The address bar supports URL navigation, search queries, paste, keyboard focus, selection/editing and appropriate history suggestions.
- Clicking the address field should select the URL according to familiar browser behavior.
- Display the scheme and hostname clearly enough to help users distinguish the intended origin. Never hide or visually falsify a navigation destination.
- Suggestions must be real, attributable to local history/search or current input. Do not insert fake trending searches or promotional cards.
- Natural-language tasks can be submitted from the persistent Agent composer. Do not silently treat a normal URL or search query as an Agent task.
- Never send raw secrets, access tokens or credential-bearing URLs to the reasoning model.

[BA-URL-001] The address field always displays the canonical scheme and hostname when not editing; security state is expressed with text, not an added icon. For ordinary HTTPS with no reported certificate/security error, display the canonical HTTPS origin without a “secure” badge. For HTTP, display `Not secure — HTTP`. For certificate errors, show `Certificate error` and block Agent actions on that origin. If the pinned CEF build exposes trustworthy evidence of active mixed content or a downgraded connection, display `Connection warning` with a concise explanation; if the build cannot reliably distinguish those states, do not invent a warning level—show the available browser security state and block high-risk Agent actions. Use the URL canonicalization and IDN spoof-display behavior of the pinned Chromium/CEF implementation or a compatibility layer maintained against that exact engine version; never write a home-grown URL canonicalizer. Follow Chromium's IDN display rules: suspicious mixed-script/confusable host labels display punycode (`xn--…`) and a text-only explanation. Test bidi controls, confusables, embedded credentials, deceptive subdomains, Unicode normalization and long hostnames. A certificate-interstitial bypass is never available to the Agent; a human-only override, if supported by CEF, requires presenting the exact hostname and typing it verbatim, then recording a redacted audit event. The default product path is to return to safety, not bypass the error.

[BA-TABSTATE-001] Loading, navigation-pending, crashed and media-playing states use text, not an additional app-owned icon or ornamental spinner: `Loading`, `Navigation pending`, `Crashed — Reload`, and `Audio playing` are accessible tab-status labels. Do not replace a page title with the status label; show status as a suffix or in the tab's accessible description, truncating the title only under the existing full-value disclosure rule. Download progress is a textual state in Downloads, for example `filename — Downloading — 42%`; progress is rendered only from CEF download events and must not create a fake tab progress animation. Tabs never steal OS focus to announce status.

[BA-INPUT-001] Do not ship network spellcheck, per-keystroke search suggestions, remote dictionary lookups or cloud writing assistance. In the initial release, first-party address-bar and Agent-composer spellcheck is disabled unless a reviewed, locally bundled dictionary has passed offline behavior and licensing tests. Search-provider autocomplete is off by default. Enabling it requires an explicit Settings opt-in explaining that query fragments are sent to the selected provider after a debounce; the UI must make the setting visible and easy to revoke. No suggestion request occurs in Private Windows, and suggestions may not be derived from private history or normal-profile history while a Private Window is active.

### 2.6 Allowed app-owned iconography

The entire first-party app icon whitelist is:

1. Undo.
2. Redo.
3. Reload/Stop, when the real browser state justifies it.
4. Bookmarks.
5. Downloads.
6. Three-dot overflow menu for Settings and less-frequent browser/application commands.

No other app-owned icon controls or decorative glyphs may be introduced without an explicit revision to this specification. This includes icons beside every heading, navigation item, card, task state, settings group or error message. Use visible text labels instead.

- All six permitted icon controls need accessible names and real behavior.
- Provide tooltips for permitted icon-only controls where discoverability benefits from one; the tooltip must not be the only accessible label.
- Do not install an icon library solely to populate the interface.
- Do not use emoji as functional UI icons.
- Remote websites retain their own rendered imagery and icons.
- Text glyphs such as a site's actual favicon, an ordinary textual bullet, or punctuation are not a reason to introduce a new icon family.

### 2.7 Undo and Redo semantics

[BA-UNDO-001] Undo and Redo use a unified app-owned action history, but they are not universal rollback controls for arbitrary websites. The toolbar Undo first follows the native undo stack when a supported editable text control has focus. Otherwise, it may compensate the most recent app-owned operation only when that operation has a recorded precondition, verified postcondition, stable target identity and a tested compensating operation. This includes an Agent `REVERSIBLE_WRITE` only when Section 13.2's reversibility proof passed. A website label such as “draft” or “edit” alone never makes an action undoable.

Executing Undo is itself a fresh, typed action: pause Agent dispatch first if a task is active, re-observe the current page, verify that the expected post-state still holds, re-run policy authorization and verify the compensation result. If the target/state drifted, the compensator is missing, or external effects may remain, disable Undo and explain why. Undo must never claim to reverse a payment, send, publication, permission grant, credential disclosure or other `IRREVERSIBLE` action.

Redo is enabled only for an operation that Browser-Agent itself just compensated and whose original preconditions can be re-established. Redo must re-observe, re-authorize and verify; it must not blindly replay a stale action. Ordinary browser Back/Forward remain navigation commands and are not aliases for Undo/Redo. Native text-entry undo remains available according to platform conventions.

### 2.8 Three-dot menu and Settings

- The three-dot control opens a familiar, labelled overflow menu.
- Settings is clearly named in the menu; it is not represented by another gear icon.
- The menu may include real browser commands that are not suitable for the toolbar: Settings, History, New window, New Private Window, Print, Clear browsing data, About, and other implemented commands. Only include commands whose corresponding runtime behavior is implemented; private-window behavior must follow Section 2.16.
- Only implemented options may appear. No dead menu item or decorative placeholder.
- Use menu grouping and textual labels, following familiar desktop conventions.

### 2.9 Persistent Agent panel

The Agent panel is a core region of the product, not an optional tool. It remains visible in all normal browsing states, including idle, model-unavailable, task-complete, error and manual browsing. Its geometry is app-controlled, not user-resizable. Only one task can be active process-wide; while a task is active the composer is disabled everywhere until that task terminates or is cancelled. In a Private Window it remains visible but displays the explicit status “Agent unavailable in Private Windows”; task submission and credential assistance are disabled there.

The panel contains:

- A concise heading such as “Agent”.
- A task composer with a clear prompt label and Submit/Run action.
- A truthful readiness/working/paused/waiting/verification/completion state.
- The current task when one exists.
- The current action in plain language, where the runtime has one.
- A compact ordered activity list of real events; not a model transcript or internal reasoning dump.
- Clear Pause/Resume and Take over controls when appropriate.
- A visible indication when user confirmation is required.
- A real result summary and verification outcome when the task terminates.

When idle, show the composer, actual readiness status and a concise instruction. Do not fill the idle panel with fabricated suggestions, fake history, marketing copy or empty decorative cards.

The task composer must distinguish task submission from ordinary address-bar navigation. During active work, retain the user's task and show actual state. The panel must never hide the website, block browser controls or falsely imply the Agent is working.

#### 2.9.1 Task-state-to-copy contract

[BA-STATE-001] Map Rust task states to user-facing status using this table; frontend copy must not be independently invented by each implementation.

| Runtime task state | Primary user-facing label | Meaning/constraint |
|---|---|---|
| `CREATED` | `Task received` | Request is recorded; no action is implied. |
| `NORMALIZING` | `Preparing task` | Clarifying task scope and allowed data, not yet acting on the page. |
| `OBSERVING` | `Inspecting page` | Collecting current page evidence; not yet executing an action. |
| `PROPOSING` | `Preparing next step` | Producing a bounded typed proposal; never expose hidden reasoning. |
| `POLICY_CHECK` | `Checking permission` | Trusted policy is evaluating the proposal; no execution until authorized. |
| `EXECUTING` | `Performing action` | A specifically authorized action is being dispatched. |
| `VERIFYING` | `Checking result` | Evaluating task postconditions using evidence. |
| `WAITING_FOR_USER` with reason `ACTION_APPROVAL` | `Waiting for your approval` | Shows the exact pending bounded action and Approve/Cancel controls. |
| `WAITING_FOR_USER` with reason `USER_INPUT` | `Needs your input` | Explains the unresolved question; no action dispatch. |
| `WAITING_FOR_USER` with reason `AUTHENTICATION` | `Waiting for you to sign in` | Manual user control is required; no credential is inferred from page text. |
| `WAITING_FOR_USER` with reason `TASK_BUDGET` | `Task budget reached` | Shows the exact ceiling reached, consumed/remaining budget and `Finish with current results`, `Extend task budget` and `Cancel task` controls. Extending budget never bypasses policy or hard resource stops. |
| `USER_TAKEOVER` | `You're in control` | Agent dispatch is stopped and pending authorizations are invalid. |
| `PAUSED` | `Paused` | No new actions are dispatched until explicit Resume. |
| `BLOCKED` | `Blocked by policy` | Policy prevented the proposed action; this is not the same as a website failure. |
| `FAILED` | `Task failed` | Runtime could not safely continue; the outcome may remain unknown. |
| `CANCELLED` | `Task cancelled` | Task was stopped by user or policy; not automatically a failure. |
| `COMPLETED` | `Task ended` | Orchestration reached a terminal end; this label does not imply success. |

Capability readiness (for example, `Model not installed`) is a separate field from task lifecycle and may be shown when no task is active. Verification outcome is another separate field governed by Section 15.5. Do not merge the three into a single status enum.

[BA-READY-001] Model capability readiness is a separate typed field with these canonical values and copy. Controls appear only when their underlying operation exists.

| Readiness state | User-facing label | Allowed action/behavior |
|---|---|---|
| `NOT_INSTALLED` | `Model not installed` | `Install model` opens an explicit, cancellable installer; task remains unexecuted. |
| `INSTALLING` | `Installing model` | Show measured download/verification progress; `Cancel` cancels work and removes or quarantines the partial artifact. |
| `INTEGRITY_FAILED` | `Model failed its integrity check` | `Retry verification` or `Remove model`; never load this artifact. |
| `INCOMPATIBLE` | `Model not compatible with this computer` | Show the measured reason (unsupported backend, RAM profile, format or runtime build); no automatic replacement. |
| `LOADING` | `Loading model` | Show real elapsed time and last runtime event; cancel/park only through a real supported operation. |
| `READY` | `Model ready` | Accept tasks only after runner health, schema and benchmark-profile checks pass. |
| `PARKED` | `Model paused to free memory` | `Resume model` triggers a bounded re-warm attempt and repeats readiness checks. |
| `RUNNER_FAILED` | `Model stopped unexpectedly` | Record the cause; offer restart for the next task only after runtime/health validation. Do not silently restart mid-task. |
| `RESOURCE_STOPPED` | `Model stopped to protect system performance` | Preserve the safe state, stop inference and show measured resource cause; manual browsing continues. |

Capability readiness, task lifecycle, task budget and verification outcome are four independent fields. A readiness failure must not be rendered as task failure if the task was never started; keep the unsent composer text and offer only actions supported by the actual readiness state. [BA-READY-001]



#### 2.9.2 Composer, disclosure and task-budget behavior

[BA-COMPOSER-001] The Agent composer is multi-line, accepts at most 8,000 Unicode code points and warns before submission if it exceeds the limit; never silently truncate task instructions. Drafts are stored locally only in the normal profile context, cleared after successful task creation by default, excluded from telemetry, and removed by the relevant clear-data action. No private-window composer draft is retained because Agent submission is disabled there. Placeholder copy is static and contains no fabricated example task.

[BA-TRANSPARENCY-001] Before the first task and through a persistent `What the agent will see` disclosure, explain that the local model may receive the task text, current page origin/title, bounded visible text relevant to current elements, accessibility labels/roles, relevant relationships, and a redacted summary of prior task steps. It does not receive vault values, cookies, session tokens, authorization headers, raw scripts, raw network bodies or unrestricted raw HTML. Before prompt assembly, the trusted input boundary scans task text for known vault-value fingerprints and high-confidence password/OTP/token patterns; matched secret material is masked before model inference and is not copied into task history. If the task appears to rely on a literal secret, ask the user to use the vault/secure-fill pathway instead of sending that literal to the model. The website itself may receive values the user authorizes the Agent to submit; the browser must separately show the destination and egress decision when that risk is material. Keep a human-readable local policy manifest reachable from the blocked-state detail view; display a plain-language block reason and requirement/policy rule reference without exposing hidden reasoning or secret content.

[BA-BUDGET-001] The Agent panel always shows remaining task budget fields while a task is active: action attempts, Agent-initiated top-level navigations, cumulative inference input/output tokens, and active execution time. Waiting for user input and an explicit pause do not consume active execution time, but their elapsed wall-clock duration remains visible separately. Budget exhaustion transitions to `WAITING_FOR_USER` with `wait_reason=TASK_BUDGET`; show exact consumed/remaining figures and offer `Finish with current results`, `Extend task budget`, or `Cancel task`. A single extension per task may raise the exhausted budget ceiling by at most 50%, shows the exact new ceiling before approval and is logged. It never overrides a policy refusal, security hard stop, no-autonomous-irreversible-action rule or hard memory ceiling.

The composer includes an optional, default-off checkbox `Only use sites I've already opened`. When enabled, Agent-controlled top-level navigation to an origin not present in the approved set captured at task start requires explicit user approval; opening a new origin is not automatically allowed. When disabled, normal multi-site research may proceed subject to all egress and action policy checks. The toggle constrains Agent-initiated top-level navigations and actions; it does not falsely promise to block arbitrary subresource requests or JavaScript network activity performed by a site itself.

A submitted task initially owns its originating tab. Agent-created tabs in that same window join the task-owned tab set only after the popup/navigation policy permits creation. The Agent may not act in an unrelated tab or another window without explicit user grant. Show a compact textual list of tabs currently in the task scope, with active/last-touched provenance. On completion, offer a real `Close task-created tabs` action that closes only tabs created by the task, never the pre-existing originating tab. Task budget/status labels must use real runtime events; an active operation also displays elapsed time and last-event time, not an indefinite animated spinner.

### 2.10 Activity and details

- Show concise task steps such as “Opening search results”, “Selecting the second result” or “Waiting for confirmation” only when these represent real runtime state.
- Do not show chain-of-thought, hidden model reasoning or raw prompt traces.
- Advanced diagnostic detail belongs in a deliberate secondary view, not the default Agent panel.
- Event ordering and timestamps must come from the orchestrator/runtime; the frontend must not invent them. Show elapsed active-task time, waiting/paused wall time and the last event timestamp from runtime events. The audit uses UTC timestamps; the UI formats wall-clock timestamps for the selected UI locale and uses a monotonic clock for elapsed durations.
- Large task/event histories should be virtualized and loaded on demand.

### 2.11 Agent pointer/target visualization

- Do not show a fake desktop mouse cursor or move the user's physical pointer.
- If the target interaction needs a visual explanation, show a restrained target outline and short textual status tied to the current, validated CEF browser action and observation epoch.
- No decorative trailing cursor, continuous pulsing, glow, animated rings or game-like HUD.
- Never render a target highlight for a stale or invalidated element.
- Redact sensitive-field contents before information reaches normal UI rendering. A visual mask is not a security boundary.

### 2.12 Human intervention, approval and handoff

[BA-APPROVAL-001] Only one approval request may be pending application-wide. It is tied to one task/action ID and has a maximum lifetime of **5 minutes**. Approval expires earlier if target, origin, frame, parameters, observation epoch or relevant page state changes. At expiry, clear the pending approval, record the expiry, return to fresh observation and recompute whether a new request is appropriate. Do not keep an old approval actionable or automatically renew it. The user must review the new structured request again.

All intervention UI is generated from structured trusted runtime data, not untrusted model prose. The UI must distinguish these two different cases:

**Approve one bounded reversible action**

- Show the exact action, target/origin, relevant non-secret values/parameters, why approval is required and the expected effect.
- Use explicit `Approve action` and `Cancel` controls. Default keyboard focus to `Cancel`, never Approve. Approval binds to that action ID and expires under BA-APPROVAL-001.
- Before approval, display an expandable `What Undo will attempt` explanation if a tested compensating operation may be used later. If the compensating operation itself requires approval, request a separate fresh approval with its own action ID and 5-minute expiry.
- After approval, Rust re-observes the page, re-runs policy and issues a single-use authorization. Approval does not bypass policy or turn an irreversible/unknown action into an allowed action.

**Hand the browser to the user for an irreversible or unknown action**

- Show the exact target/origin, action the user may perform, relevant parameters and known consequences.
- Use `Take over` and `Cancel task`; never show a generic Confirm button that could be mistaken for authorizing the Agent to submit.
- The Agent stops dispatching browser actions and gives the user control of the real CEF page. The user performs the consequential final action manually.
- When the user has finished, the Agent stays stopped. It resumes only after an explicit Resume request, fresh perception, reconciliation of current page state and policy revalidation.

[BA-INTERRUPT-001] Pause, Cancel and Take over stop new action dispatch immediately, but acknowledge that a native browser action already dispatched may not be cancellable or reversible. If an action is in flight, show `Finishing current step, then stopping` and wait only to its bounded action deadline. Then reconcile the page. If evidence cannot establish whether a side effect occurred, preserve `UNKNOWN`; never tell the user an operation stopped before execution if it may already have reached the page. Cancellation tokens interrupt waits and inference immediately where supported. An operation that exceeds its deadline enters the defined hard-stop/recovery path. While `WAITING_FOR_USER`, never steal OS focus; show in-app status and use OS notifications only if the user explicitly enabled them. On system sleep/suspend, pause the task, expire pending approvals, invalidate transient targets and on wake re-stabilize/re-perceive and disclose page drift before resume.

Cancel must not execute a partial version of the prohibited action. If the browser state changes while approval is pending, invalidate the approval and show a fresh review. The Agent panel stays visible and says exactly whether it is awaiting approval, handing over control, pausing after a current step, paused or ready to resume.

### 2.13 Secondary screens

History, Bookmarks, Downloads, Settings, Privacy & Security, Agent settings, Credentials/Vault, Model status, Network policy, Quarantine and Audit/Diagnostics may be separate views reachable through real menus or links. They must retain the same design tokens and plain-language interaction model. Add distinct `Agent task history`, `Research notes`, `Skills`, `Keyboard shortcuts`, and `Model integrity/readiness` views. Do not conflate Agent task history with visited-page history. Task history offers `Run a similar task again`, which only pre-fills the composer and never submits automatically. Private-window activity is not written into Agent task history or research notes.

- The default browser view must not become an event ledger.
- Technical identifiers such as epochs, frame IDs, target IDs, CEF callback names, renderer-frame IDs and raw policy codes belong only in diagnostic detail.
- Credential content and secret material must never be shown in general settings or logs.
- Settings controls must map to real runtime behavior, not visual-only toggles.
- A `Blocked by policy` detail view shows the plain-language reason, the applicable human-readable policy rule, what was not executed and safe next choices; do not expose page secrets or model internals.
- Task reports are user-initiated local exports using the schema defined in BA-HISTORY-001. Preview/redact before writing/exporting; never upload automatically.
- `Research notes` supports local-only, source-attributed excerpts; `Skills` supports inspect/review/disable/delete with provenance; `Model` supports measured readiness and integrity checks. Add `Export Browser-Agent data` with a category preview and redaction step. The default bundle may include bookmarks, selected normal-profile history, settings, notes, skills, Agent task summaries and retained audit summaries. It excludes vault secrets, cookies/session storage, raw page DOM, unredacted prompts, model weights and downloaded files unless each category is separately selected through a dedicated risk-reviewed export flow. Export writes locally and never uploads.
- Task activity shows whether a tab is `Agent active` or `Last used by Agent`; the tab itself must not claim active ownership if the task is paused, ended or bound to another tab.

### 2.14 Multi-window behavior

[BA-WIN-001] Browser-Agent supports multiple native top-level application windows because New Window is a standard browser command. Each window owns its own tab strip, address state, active tab, Agent panel and window lifecycle; normal windows share the single persistent normal browsing profile in this release; creating/selecting multiple normal profiles is deferred under Appendix F. A Private Window always uses its separate ephemeral context. Closing a window must not implicitly close other windows.

To preserve the 8 GB reference-device memory and prevent conflicting autonomous control, **only one autonomous task may be active across the entire Browser-Agent process at a time**, not one per window. The active task is bound to its originating window and tab. A task submitted from another window while one is active is not silently run or queued: show “Another task is running in a different window” and offer only real actions to focus that window or cancel the current task. If the owning window/tab is closed or replaced, invalidate pending authorizations, stop dispatch, preserve a safe checkpoint and resolve the task via the defined pause/cancel/recovery protocol. Manual browsing remains available in all other windows.

Add platform-appropriate New Window shortcuts (Command + N on macOS; Control + N on Windows), test concurrent window creation/closure and background-window lifecycle, and ensure browser profile locking prevents a second process from corrupting the same profile.

[BA-WIN-003] If a task is already active, the composer is disabled in both the owning window and every other window; there is no hidden queue. The owner window says `One Agent task is already running. Finish or cancel it before starting another.` Other windows say `Another task is running in a different window` and offer only `Focus task window` and `Cancel current task` where valid. Task-owned tabs remain in the owning window; moving, tearing off or closing a task-owned tab is blocked while automation is active unless the user explicitly cancels/takes over first. Tabs may be torn off when no task is active. Closing the owning window is a safe stop as defined in Sections 2.14 and 15.6.

### 2.15 Third-party browser extensions

[BA-EXT-001] Third-party browser extensions, extension stores, Chrome Web Store installation, arbitrary user extension packages and extension-driven native privileges are **not supported** by this product contract. Do not expose an Extensions page or install surface, and do not emulate extension support through unrestricted script injection. This is a deliberate product security and compatibility boundary, not an unfinished visual screen. CEF does expose a `CefRequestContext::LoadExtension` API and supports a subset of Chrome extension APIs, so it is inaccurate to claim CEF cannot load extensions. The current CEF JavaScript-integration page separately marks `CefRegisterExtension`'s legacy custom V8-extension mechanism as deprecated in M154; that is not the same thing as Chrome-style extension package support. This product deliberately does not ship third-party extensions because CEF's supported API subset, cross-platform compatibility requirements and extension privilege surface need a separate threat model and validation effort. Revisit only through a reviewed scope revision and architecture decision record.

First-party Browser-Agent features must be implemented as audited product code with least privilege. Any future extension capability would require a separate sandbox/permission design, per-origin grants, an update/signature policy, a dedicated adversarial test suite and explicit scope revision before implementation.

### 2.16 Private browsing windows

[BA-PRIV-001] Provide a “New Private Window” action using a dedicated ephemeral CEF request context with no persistent per-profile cache path. Do not share cookies, local storage, cache, history, form history or session state with the normal profile or another private window. Close the private window by destroying its request context and removing its remaining ephemeral storage; verify the behavior through disk-artifact and cross-context isolation tests for the pinned CEF release. Installation-wide CEF resources may exist, but website browsing records must not be written to the normal profile.

Private-window guarantees and limits must be stated plainly: sites and networks can still observe requests, downloaded files remain on disk unless the user removes them, and the operating system may retain its own records. Private history and private address suggestions are never added to persistent browsing history. Saving a bookmark requires an explicit action and a notice that the bookmark is persistent. Password autofill and password saving are disabled in private windows. The Agent is **disabled in private windows** for this release so page context, prompts, actions, task history and audit evidence cannot accidentally cross privacy modes. The user can still browse manually.

---

[BA-PRIV-002] Private Window new-tab content is a stateless local page. It may not read or suggest normal-profile history, bookmarks, notes, autocomplete records, Agent tasks, saved forms or model prompt history. Do not persist a private address-bar value or form value after the private context closes. Disable both password and non-password form/address autofill in Private Windows; clipboard actions remain manual user actions and are never exposed to the Agent. Do not restore private tabs after crash/restart. Explicitly chosen downloads and user-saved bookmarks can persist only after clear disclosure that these artifacts outlive the private window.

## 3. Typography, readability and responsive behavior

### 3.1 Serif-first typography contract

- Use a carefully selected, locally bundled serif family across all first-party Browser-Agent UI: tabs, address bar, Agent panel, settings, dialogs, menus, status labels and error messages.
- Do not use sans-serif as a silent fallback. Do not inherit `Inter`, `Roboto`, `Arial`, `system-ui` or foreign font variables as the intended design.
- Remove prior `--kimi-font-*` dependencies unless an independently verified project requirement proves they are necessary. The default should be project-owned font tokens.
- The display/body distinction may use weights, sizes and spacing of the same serif family. A second family is not needed unless a documented accessibility or rendering issue makes it necessary.
- Monospace is not a default UI font. Use it only for genuine technical data in secondary diagnostics, if at all.
- Select the actual font by inspecting repository assets, verifying its license, testing rendering at small sizes and bundling it locally. Do not claim a family is bundled until the file exists and the build proves it is loaded.
- Serif-first is a product identity decision that deliberately differs from common sans-serif UI defaults. The chosen serif must be highly legible at small sizes. If it fails the tests below, select a better serif; do not silently fall back to sans-serif.
- [BA-TYPE-001] The selected serif family must support tabular numerals (`font-variant-numeric: tabular-nums`) for timestamps, budgets, percentages, IDs and aligned tables. If its numerals are not tabular or remain visually ambiguous, choose another licensed serif family or a separately bundled licensed serif numeric face; do not use a sans-serif fallback. URLs, hostnames and security-sensitive strings must pass the Section 2.5 canonical-display/IDN tests regardless of typeface.

### 3.2 Typography scale

Use a small coherent scale that can adapt to operating-system scaling. Suggested starting tokens:

- Tab strip height token: `--ba-tab-strip-height: 34px` (CSS logical pixels; adapt only for supported text scaling).
- Navigation toolbar height token: `--ba-navigation-toolbar-height: 44px` (CSS logical pixels; includes the address field and text-labelled navigation controls).
- Compact browser labels: 12–13 CSS px, only when necessary and still legible.
- Standard toolbar and compact control text: 13–15 CSS px.
- Agent activity and general UI text: 14–16 CSS px.
- Longer body/help text: 15–17 CSS px.
- Secondary screen headings: 18–24 CSS px, depending on hierarchy.

These are initial design tokens, not fixed physical-pixel promises. Validate at 100%, 125%, 150% and OS text scaling where available. Do not shrink essential labels to preserve a rigid layout; adapt the layout instead.

### 3.3 Readability requirements

- Ensure long URLs, page titles, error messages, timestamps and numeric values remain legible.
- Prefer concise text, sentence case and familiar vocabulary.
- Avoid all-caps paragraphs, compressed tracking and very light weights.
- Maintain comfortable line height for multi-line text and enough spacing to distinguish separate tasks.
- Use truncation only when necessary and provide a way to inspect the complete value. Do not truncate origin/security information in a misleading way.
- Support keyboard navigation, readable focus, high-DPI rendering and resizing.
- Test with actual website titles and URLs of realistic lengths, not just short demo labels.

### 3.4 Responsive desktop support

The supported product is a desktop application for macOS and Windows, not a claimed mobile browser. “Responsive across devices” means the desktop shell remains readable and functional across supported laptop/desktop sizes, display scales, external monitors, full-screen and resized windows.

Minimum supported application **client area**: **900 × 640 CSS logical pixels**, measured inside the OS-owned title-bar/frame region. The native host must translate this client-area minimum to the correct outer-window constraint per platform; do not count the OS title bar twice. Enforce this minimum so resizing cannot make core controls unusable.

Test at minimum:

- 900 × 640 logical pixels (minimum supported client area, excluding OS-owned frame/title bar);
- 1024 × 768 logical pixels;
- 1280 × 720 logical pixels;
- 1440 × 900 logical pixels;
- 1920 × 1080 logical pixels;
- high-DPI and 125–150% OS scaling where available.

[BA-LAYOUT-001] At widths below 960 logical pixels, stack the browser viewport above the persistent Agent workspace. At or above 960 pixels, use the side-by-side layout unless text scaling or the native accessibility setting requires stacking earlier. Geometry at the minimum 900 × 640 client area is a normative box-model contract: 34 px tab strip + 44 px navigation toolbar + 360 px browser viewport + 200 px Agent workspace + two 1 px separators = 640 px. Do not add unbudgeted app-owned vertical padding or a status bar. The 360 px browser viewport is the tested floor, not a promise that every site will be comfortable at that height; include cookie-banner, sticky-header, login, form, long-page and responsive-site usability checks at this exact size. The 200 px Agent region must keep the composer, primary task state and Pause/Take over/Resume or required confirmation controls reachable; activity history scrolls independently and may be visually abbreviated without hiding required controls. If real-device usability tests show that this floor is not viable, raise the minimum window size by a documented measurement-backed revision—do not silently reduce the viewport or hide the Agent.

The Agent panel remains visible and must not become a modal, hidden tab, dismissible drawer or settings toggle. Both browser and Agent regions scroll independently where appropriate. Never overlap their active hit targets, clip the address bar, hide confirmation controls or allow horizontal scrolling of the app shell.

[BA-LAYOUT-002] The Agent panel is app-sized and is not user-resizable in this release. In stacked mode, set Agent-region height to `clamp(200px, 30vh, 400px)` measured against the application client area, with 200 px as the hard floor and 400 px as the ceiling; the browser viewport receives remaining vertical space after the tab strip, toolbar and separators. At the 900 × 640 minimum, the rule resolves to the specified 200 px Agent area and 360 px browser area. Recalculate geometry whenever OS text scaling requires a stacked layout.

[BA-LAYOUT-003] At the 960 px side-by-side breakpoint, the English first-release toolbar must fit Back (minimum 42 px), Forward (minimum 58 px), six permitted icon controls (32 px each), at least 180 px of editable address field, and 6 px gaps without mid-word truncation. Below 960 px use stacked mode. In future localized builds, recompute this minimum from the longest reviewed locale strings and stack earlier when the measured toolbar minimum cannot fit; never truncate a control label mid-word or hide origin/security information. Pseudo-localization tests must include 30–50% text expansion, long German/Finnish/Portuguese labels and RTL.

[BA-THEME-001] First-party UI follows the OS appearance setting automatically. Dark appearance uses the documented Deep Navy/Deep Teal foundation; light appearance uses approved Paper/Ink and contrast-tested semantic derivatives. The seven canonical brand swatches remain immutable; theme changes only use approved semantic mappings/derivatives, never per-component ad hoc hex values. No app-specific appearance toggle ships initially. Remote webpages retain their own theme behavior. Every required Section 21.1 screenshot and contrast test runs in both OS-following appearances.

### 3.5 Text resizing and overflow

- Use flexible layout constraints rather than fixed widths everywhere.
- Avoid absolute positioning for text-heavy layout.
- Permit tab titles and task descriptions to truncate with accessible full-value disclosure.
- Use `min-width: 0`, sensible overflow rules and grid/flex constraints where needed.
- Prevent the persistent Agent panel from forcing the browser region below its usable minimum; change the layout rather than allowing the regions to overlap.
- Ensure page zoom and website content are controlled by Chromium and do not accidentally scale the app shell.

### 3.6 Localization, Unicode and writing systems

[BA-I18N-001] The first release ships English UI copy, but the code and layout must be localization-ready. All first-party UI copy comes from keyed message resources; do not concatenate sentence fragments or bake English grammar/order into components. Use locale-aware date, time and number formatting. Store text as Unicode, support IME composition and test CEF keyboard/input-method handling in the native surface.

The canonical visual identity remains serif-first. The chosen primary font must be locally bundled, licensed and checked for glyph coverage. If it does not cover a required script, use an explicitly configured, locally bundled, licensed script fallback with correct shaping rather than showing tofu boxes or forcing Latin transliteration; document and test that exception. Font fallback must not silently load from a network CDN.

Use CSS logical properties (`inline-start`/`inline-end`, `margin-inline-*`, `padding-inline-*`) for layout wherever direction-neutral geometry is intended. Support bidirectional URLs and mixed-script content without visually falsifying origin information. Before declaring localization-ready, run pseudo-localization/expansion tests and test the shell, menus, tab titles, forms, error states and Agent panel with Arabic/Hebrew RTL samples, CJK, Devanagari, combining marks and long translated strings. The first release does not claim translated UI until a complete reviewed locale pack exists. The URL bar additionally follows Section 2.5's Chromium-aligned IDN/punycode display policy; RTL layout must not reverse or visually reorder the actual hostname. Run bidi-control, mixed-script confusable and combining-mark hostname fixtures on both platforms.

---

## 4. Anti-AI-slop visual and UX rules

### 4.1 General rule

A visual element must earn its place by improving hierarchy, usability, orientation, accessibility or truthful system feedback. Decorative complexity is not premium quality.

### 4.2 Prohibited patterns

Do not ship:

- purple/blue, cyan/purple or other decorative gradients;
- glow effects, neon borders, luminous orbs, animated light or cursor-following effects;
- glassmorphism, backdrop blur or translucent layers used only as decoration;
- generic AI landing pages, oversized empty heroes or marketing content inside the product;
- repeated rounded-card grids, pill-shaped everything or floating bubble layouts;
- decorative sparkle/magic-wand motifs, emoji controls, AI badges or meaningless icons;
- fake statistics, placeholder testimonials, fabricated history, sample tasks presented as real or fake success metrics;
- artificial progress rings, activity charts, decorative gauges or nonfunctional status lights;
- shimmer skeletons, typing dots, confetti, bouncing controls or looping animation;
- excessive shadows, ornamental dividers, floating illustrations or blank space that serves no task;
- raw engineering metadata in the normal browser/Agent view;
- inert controls, fake browser content, simulated browser-runtime behavior or simulated task completion;
- an interface that reads as a SaaS operations dashboard or an AI chat product.

### 4.3 Required alternatives

- Use alignment, typography, spacing, flat color fields and deliberate borders to create hierarchy.
- Use text labels for unfamiliar actions.
- Use accent color only for meaningful selection, focus, primary action or true runtime state.
- Use compact plain-language task updates instead of artificial conversational animation.
- Use clear messages for loading, failure, interruption, confirmation and verification.
- Use one restrained brand motif only if it improves identity and never as a replacement for control labels.

### 4.4 Plain-language writing

Avoid promotional or vague phrases such as “Unlock your productivity,” “Experience the future,” “Supercharge your workflow,” “Magic,” “Autopilot everything” or “AI-powered insights” unless there is a specific user-understood meaning. Do not expose internal implementation terms such as “fallback” in ordinary user-facing copy; describe the exact behavior or next step instead. Do not add a “Live now” button, pricing/payment option or generic marketing call-to-action without a real, approved product function. Describe the actual action, state or next step. Examples:

- “Waiting for your confirmation” instead of “Almost there.”
- “Could not identify the requested button” instead of “Something went wrong.”
- “The result could not be verified” instead of “Done.”
- “Model not installed” instead of “AI unavailable” when that is the precise cause.


[BA-COPY-001] Every user-facing error follows the same order: (1) what was attempted, (2) what actually happened, (3) what is known/unknown now, and (4) the safe next action. Use plain sentence case and verb-first action labels. Error identifiers and stack details appear only in diagnostics. Add worked copy examples for navigation, CEF/model startup, policy block, verification ambiguity, download rejection and resource stop. Keep UI strings in keyed resources and run a CI copy lint that flags marketing filler (including `magic`, `supercharge`, `AI-powered`, `unlock productivity`, unsupported superlatives, fake progress, and unlabeled “Done”) for review. The lint may use an explicit allow-list for technical/product names; it may not change displayed strings automatically.

[BA-FIND-001] Provide native-backed Find in Page. The visible menu entry is `Find in page` with Command + F on macOS and Control + F on Windows. Search is local to the active CEF page, highlights matches using browser-native functionality, supports next/previous match and close, and does not send search text to Browser-Agent services. Add a text-only `Keyboard shortcuts` screen to Settings. No additional toolbar icon is introduced.

[BA-SOUND-001] Browser-Agent emits no app-owned UI sounds by default and has no sound-only indication of Agent activity or state. Website audio remains under website/browser permission and user controls. Tab audio state is textual under BA-TABSTATE-001.

---

## 5. Apple-informed craft and platform principles

Use Apple's official Human Interface Guidelines as a quality reference—not as a requirement to copy Apple's current visual design. Apply the principles of purpose, agency, flexibility, simplicity and craft. Browser-Agent's distinctive visual identity remains serif typography, an earthy seven-color palette, opaque surfaces, purposeful geometry, selective corner rounding and almost no motion.

### 5.1 Purpose and simplicity

- Start with the user's task and the most important content.
- Remove elements that do not help the user understand or act.
- Keep browser content, address navigation and the persistent Agent composer easy to identify.
- Avoid adding controls because a template contains them.
- Avoid unnecessary modal steps and nested settings levels.

### 5.2 Agency and predictability

- Users can pause active work, take manual control and cancel a pending action.
- Controls act on the context they appear to control.
- Current tab, browser URL, Agent task, target and verification outcome have one authoritative runtime source each.
- Preserve familiar browser and desktop conventions, including keyboard behavior, menus, focus handling and window resizing.
- After user takeover, the Agent must not resume without an explicit action and fresh perception.

### 5.3 Craft and consistency

Treat alignment, spacing, border placement, typography, loading behavior, error states and interaction timing as engineering quality—not decoration. Make repeated controls behave the same. Test the entire product at realistic window sizes and with real content. Iterate from rendered screenshots and runtime behavior, not source-code aesthetics alone.

### 5.4 Platform fit without visual imitation

- Respect platform window management, file dialogs, secure credential features, keyboard conventions and accessibility APIs.
- Preserve recognizable desktop-browser behavior on both macOS and Windows.
- Do not copy Liquid Glass, blur, translucency, system-wide rounding, or Apple's particular icon style simply because Apple uses them.
- Do not redraw OS-owned title-bar buttons as a custom brand surface.
- In Browser-Agent-owned UI, never manually render `⌘`, `⌥`, `⇧` or `⌃` shortcut glyphs. Use text labels such as `Command + L`, `Control + L`, `Shift + Enter`. Platform-specific shortcut behavior can still be correct.

### 5.5 Icons and menus

Apple's guidance favors familiar controls, deliberate toolbars and purposeful icon use. Browser-Agent has an even narrower explicit icon whitelist defined in Section 2.6. Keep infrequent commands in the three-dot menu with clear text labels. Do not add one icon to every menu row, settings group or activity step.

### 5.6 Materials and visual depth

- Prefer opaque, flat surfaces.
- No Liquid Glass, frosted surfaces, translucent cards or blur.
- No ambient or decorative drop shadows. If a native platform component uses a system-owned shadow, do not fight the platform; do not add a custom shadow to first-party UI.
- Use borders and tone changes to establish depth and grouping.

### 5.7 Motion policy

Motion is disabled by default unless a short transition clarifies a real state change. No decorative motion is permitted. Do not use GSAP or Anime.js merely because the dependencies are installed. If retained for a genuine functional transition, their use must be narrowly scoped, nonessential, reduced-motion-aware and justified in code review.

---

## 6. Color system and corrected token contract

### 6.1 Canonical seven-color palette

These seven swatches are immutable brand values. Components must use approved semantic tokens, not hardcoded hex values.

| Canonical name | Hex | Intended role |
|---|---|---|
| Crimson | `#CC2132` | Destructive/failed state and explicit danger actions |
| Tangerine | `#E6692A` | Secondary warm interaction; use with a verified foreground pairing |
| Golden Orange | `#ED9530` | Primary action, focus emphasis and working state where pairing passes contrast |
| Warm Peach | `#E9BD96` | Main warm text on approved dark surfaces |
| Olive | `#988A20` | Supporting/selected emphasis; avoid as small text on dark surfaces |
| Deep Teal | `#34685C` | Structural band, secondary surface and verified state fill |
| Deep Navy | `#1E404F` | Main foundation and architectural surface |

Do not change these values to improve a single component. Add an approved derivative and contrast-test its exact pairing if necessary.

### 6.2 Approved existing supporting values

The following existing supporting values may be retained only through semantic tokens and the corrected rules below. They are not additional canonical brand swatches.

| Token/value | Hex | Use/constraint |
|---|---|---|
| Peach Boost | `#F1D4BB` | High-contrast warm text; approximately 4.53:1 on Deep Teal |
| Crimson Hi | `#E7949D` | Error text on Deep Navy only where the pairing is revalidated; approximately 4.80:1 on Navy |
| Tangerine Hi | `#F0A57F` | Warning text on Deep Navy only where the pairing is revalidated; approximately 5.47:1 on Navy |
| Olive Hi | `#D1CA9B` | Informational text on Deep Navy; approximately 6.64:1 on Navy |
| Muted | `#C0A487` | Secondary text on Deep Navy only; approximately 4.68:1 on Navy |
| Quiet | `#A2917D` | Disabled text only where the inactive-component exception applies; never ordinary input placeholder text |
| Ink | `#16303B` | Text on approved light/paper surface; approximately 12.30:1 on Paper |
| Raised Surface | `#285255` | Elevated opaque dark surface |
| Overlay Surface | `#234652` | Opaque menu/dropdown surface; the prior comment claiming it was an 8% white blend of Navy was inaccurate and must be removed or corrected |
| Border Subtle | `#4C7A70` | Decorative separators only on tested backgrounds; not universal control boundary |
| Border Strong | `#6D928A` | Border on tested contexts; not universal on Deep Teal |
| Olive Deep | `#676C33` | Olive state fill with white label; white contrast is approximately 5.57:1 |
| Paper | `#F7F1E8` | Primary first-party surface in OS light appearance and evidence/reading surface; never force this theme onto remote websites |
| White | `#FFFFFF` | Neutral contrast exception for state/action text where required |
| Light Raised | `#EFE7DC` | Approved light-theme grouping surface; pair with Ink or tested light-theme text tokens |
| Light Secondary Text | `#5D544B` | Secondary text on Paper/White; contrast approximately 6.60:1 on Paper and 7.41:1 on White |
| Light Warning Text | `#8A3B14` | Warning copy on Paper/White; contrast approximately 6.89:1 on Paper and 7.73:1 on White |
| Light Border | `#4C7A70` | Interactive boundary on Paper/White after adjacent-color contrast validation; not body text |

Do not claim that a color value is WCAG compliant in isolation. Accessibility is a relationship between foreground and background, and applicable component boundaries must also be checked against adjacent colors.

### 6.3 Required foreground/background rules

The following calculations use the hex values above and standard WCAG relative-luminance contrast mathematics. Validate again against final rendered colors, opacity, borders and platform scaling.

| Foreground | Background | Contrast | Rule |
|---|---|---:|---|
| Warm Peach `#E9BD96` | Deep Navy `#1E404F` | 6.41:1 | Approved for normal text |
| Peach Boost `#F1D4BB` | Deep Teal `#34685C` | 4.53:1 | Approved for normal text, with little margin; avoid reducing opacity |
| Warm Peach | Raised Surface `#285255` | 5.02:1 | Approved for normal text |
| Warm Peach | Overlay Surface `#234652` | 5.89:1 | Approved for normal text |
| Muted `#C0A487` | Deep Navy | 4.68:1 | Approved for normal text on Navy only |
| Muted | Deep Teal | 2.71:1 | Prohibited for normal text |
| Muted | Raised Surface | 3.67:1 | Prohibited for normal text |
| Muted | Overlay Surface | 4.30:1 | Prohibited for normal text |
| Quiet `#A2917D` | Deep Navy | 3.62:1 | Prohibited for ordinary placeholder/body text |
| Ink `#16303B` | Paper `#F7F1E8` | 12.30:1 | Approved for normal text |
| Deep Navy | Golden Orange `#ED9530` | 4.70:1 | Approved primary-action label pairing |
| White | Crimson `#CC2132` | 5.48:1 | Approved destructive-state/action label pairing |
| White | Deep Teal | 6.40:1 | Approved state label pairing |
| White | Olive Deep `#676C33` | 5.57:1 | Approved waiting-state label pairing |

Normal text must meet at least 4.5:1 contrast. Large text may use the applicable 3:1 threshold, but components must not rely on large-text exemptions for compact controls. Applicable non-text control boundaries and focus indicators require at least 3:1 contrast against the adjacent color, subject to the WCAG criterion and its exceptions.

### 6.4 Surface-aware semantic tokens

Do not use a single `--ba-text-secondary` value on every surface. The token layer must include surface-aware roles, for example:

- `--ba-text-on-foundation` — Warm Peach on Deep Navy.
- `--ba-text-secondary-on-foundation` — Muted on Deep Navy only.
- `--ba-text-on-teal` — Peach Boost on Deep Teal.
- `--ba-text-secondary-on-teal` — use an independently verified light warm derivative that passes 4.5:1; do not use Muted.
- `--ba-text-on-raised` — Warm Peach or Peach Boost on Raised Surface.
- `--ba-text-on-overlay` — Warm Peach or Peach Boost on Overlay Surface.
- `--ba-text-on-paper` — Ink on Paper.
- `--ba-text-secondary-on-paper` — an independently verified dark neutral, such as `#6D5A4B` only after validation against the actual Paper value.
- `--ba-light-foundation: #F7F1E8` (Paper); `--ba-light-surface: #FFFFFF`; `--ba-light-raised: #EFE7DC`.
- `--ba-light-text-primary: #16303B` (Ink); `--ba-light-text-secondary: #5D544B`; `--ba-light-text-warning: #8A3B14`.
- `--ba-light-border-interactive: #4C7A70` only where measured contrast against the adjacent light surface is at least 3:1; use a darker tested token if not.
- Keep action/state foreground pairings surface-aware in light mode too. Deep Navy on Golden Orange is the primary action pairing; White on Deep Teal/Crimson/Olive Deep are approved filled-state pairings. Never simply invert colors or reduce opacity to create light mode.

A token name must describe its allowed context. If a component changes background, it must select the corresponding text token rather than assuming one foreground works everywhere.

### 6.5 Surface roles

- **Foundation:** Deep Navy `#1E404F`.
- **Secondary structural surface:** Deep Teal `#34685C`.
- **Raised surface:** `#285255`.
- **Overlay/menu surface:** `#234652`, opaque unless a documented system-owned surface requires otherwise.
- **Light appearance:** Paper foundation `#F7F1E8`, White surface `#FFFFFF`, Light Raised `#EFE7DC`, Ink primary text `#16303B`, Light Secondary Text `#5D544B`, Light Warning Text `#8A3B14`, with the tested token mappings in Section 6.4.
- **Remote webpage:** preserve the website's own appearance. Do not apply Browser-Agent colors or typography to remote site content.

### 6.6 State semantics

Colors communicate real runtime state, never decoration.

| State | Semantic fill/text treatment | Requirement |
|---|---|---|
| Working | Golden Orange fill + Deep Navy text | Only while work is genuinely running |
| Waiting for user | Olive Deep fill + white text | Only for an actual wait/checkpoint |
| Verified success | Deep Teal fill + white text | Only when verification has sufficient evidence |
| Failed/blocked | Crimson fill + white text | Only for actual failure/blocking |
| Unknown | Transparent/opaque-surface background with accessible outline and text | Never present as success; explicitly say it is unknown/unverified |
| Idle/ready | Quiet neutral surface/text using approved contrast pairings | Must not imply active work |

Never rely on color alone. Each meaningful state must have readable text. “Completed” and “Verified success” are different: completion can describe task termination; verification describes evidence about the outcome.

### 6.7 Focus and borders

- Focus must be visibly identifiable without glow.
- Choose a focus token for the current surface. Golden Orange is approximately 4.70:1 against Deep Navy but only about 2.73:1 against Deep Teal; therefore it cannot be used blindly on Teal.
- Use a surface-aware light focus outline on Teal surfaces, such as Peach Boost, after confirming rendered contrast.
- Do not use `Border Subtle` or `Border Strong` as universal control borders. Their contrast on Deep Teal is too low for some non-text boundary requirements.
- Add a surface-specific high-contrast border token where a boundary is required, and document its measured ratio.
- Do not use a transparent outline or outer glow to simulate focus.

### 6.8 Color token enforcement

- Components use semantic tokens only; direct hex literals belong only in the canonical token file and approved assets/tests.
- Add lint/build checks for unapproved hardcoded colors.
- Token tests should compute ratios for all approved foreground/background combinations.
- Fail the quality gate when a normal-text pairing falls below 4.5:1, or an applicable control boundary/focus indicator falls below its required threshold.
- Keep contrast tests in CI and update their fixtures whenever a derivative or surface changes.

---

## 7. Geometry, spacing and surfaces

### 7.1 Selective corners

Use corner radii with intention—not zero radius everywhere and not rounded corners everywhere.

Recommended tokens:

- `--ba-radius-none: 0px` — main shell edges, tab strip geometry, dividers, browser viewport boundaries and most toolbar structure.
- `--ba-radius-small: 2px` — subtle tab or control details where it improves definition.
- `--ba-radius-control: 4px` — address field, task composer, standard input or compact actionable control when a slight radius improves affordance.
- `--ba-radius-surface: 6px` — occasional dialog, menu or grouped Agent surface where a soft boundary improves comprehension.
- Maximum default app-owned radius: 6px. Larger radii require an explicit UX reason and review.

Do not use pill-shaped buttons, fully rounded cards, capsule labels, giant rounded containers or a different radius on every component. If a browser-native convention needs a small corner, use it consistently. OS-owned window corners remain OS-owned.

### 7.2 Borders and separators

- Use 1 px borders by default; 2 px only for a clear focus/selection/important boundary.
- Use borders to define structure, separate the browser from the Agent, establish selected tabs and make inputs discoverable.
- Avoid borders around every paragraph or every task step.
- Do not rely on low-contrast boundaries to separate interactive controls.

### 7.3 Spacing

Use a consistent spacing scale based on 4 px:

- 4 px — tight local spacing;
- 8 px — related controls/labels;
- 12 px — compact grouping;
- 16 px — standard internal padding;
- 24 px — section separation;
- 32 px — major grouping where space allows.

Do not add large empty padding to imitate luxury. Do not compress adjacent controls until they become hard to activate. Space should reflect structure and frequency of use.

### 7.4 Elevation and shadows

- No custom ambient/drop shadows on normal product-owned surfaces.
- Use surface color, border, ordering and spacing to indicate hierarchy.
- Native system-provided shadows are acceptable where a native OS component supplies them.
- Do not introduce a shadow just to make a card appear “premium.”

---

## 8. Motion and animation

### 8.1 Default behavior

Default to immediate UI response. No decorative animation. No glow, shimmer, bounce, elastic easing, spring overshoot, parallax, pulse, looping background motion, animated gradients, confetti or cursor-following effects.

### 8.2 Allowed motion exceptions

A short, restrained transition is allowed only when it materially clarifies a real state change—such as moving the Agent region into a narrow-window stacked layout or revealing a confirmation surface. It must:

- correspond to a real state transition;
- not delay interaction;
- not be the only way to perceive the state;
- be short and visually quiet;
- work with reduced motion enabled;
- have no loop or decorative idle animation.

Suggested maximum for optional transitions: 120–160 ms. Immediate changes are preferred. Durations are ceilings, not a quota for animation.

### 8.3 Libraries

GSAP and Anime.js are not requirements. If already installed, they may remain only if a verified interaction needs them. Do not use both libraries to animate routine hover, text, every panel or status. Remove unused animation code/dependencies if they provide no legitimate product value.

### 8.4 Reduced motion

Respect `prefers-reduced-motion: reduce` throughout app-owned UI. Essential state changes remain clear when all optional motion is removed. No workflow depends on a moving pointer, animated indicator or transition to communicate that an action happened.

---

## 9. Lazy loading and performance-aware interface behavior

Lazy loading is required where it improves start-up, scrolling and memory without hiding essential information.

### 9.1 Load immediately

Do not lazy-load:

- the browser shell and navigation controls;
- tabs and active-tab state;
- the persistent Agent panel shell, composer and readiness state;
- the current browser viewport surface;
- controls required to stop, pause, cancel, intervene or confirm;
- visible content needed to understand a current high-impact state;
- the minimal runtime bridge needed for user actions. [BA-BRIDGE-001] The eagerly available bridge is the versioned, allow-listed API defined in Section 20.7 (including the documented browser navigation, Agent submit/pause/cancel/resume/takeover, confirmation and event-subscription methods); never lazy-load the bridge or security-critical validators, and never expose a generic native command dispatcher.

The product must be interactive as soon as its security-critical prerequisites are ready. Do not delay basic manual browsing while a local model or secondary screens load.

[BA-BRIDGE-001] The eagerly loaded local UI bridge exposes only the minimum typed methods needed for immediate interactions: `browser.navigate`, `browser.back`, `browser.forward`, `browser.reload`, `browser.stop`, `browser.focusAddress`, `tabs.create`, `tabs.close`, `tabs.activate`, `agent.submitTask`, `agent.pause`, `agent.cancel`, `agent.takeOver`, `agent.resume`, and `agent.respondToApproval`. Availability is state-gated; the presence of a method never grants authority. Optional history, notes, skills, model-management, diagnostics and export modules may lazy-load, but any currently active approval/cancel/takeover control must remain live in the preloaded bridge. See Section 20.7 for exact type-validation and per-message caller checks.

### 9.2 Lazy-load when appropriate

- Non-critical, below-the-fold app-owned images: use native lazy loading where applicable.
- Settings, diagnostics, audit detail, credential-vault views, long history/download views and other secondary routes: load their code on demand.
- Large task/event histories: virtualize rows and retrieve data in pages/chunks.
- Heavy charts or visualizations: do not load them unless a real secondary screen requires them. Do not add them merely for decoration.
- Nonessential assets and help content: load when near use or on demand.

### 9.3 Browser content distinction

The remote webpage is rendered by managed Chromium. Do not rewrite the website's DOM to impose Browser-Agent lazy loading, typography, styles or content behavior. Let Chromium and the website's own code govern remote-page lazy loading. Any browser-runtime interception must be justified by security/performance requirements, preserve website behavior and be tested.

### 9.4 Lazy-loading acceptance

- First paint includes usable browser controls and the persistent Agent panel.
- Basic navigation works without waiting for secondary React routes or model initialization.
- No blank placeholders or shimmer effects are shown solely because a secondary module is lazy-loaded.
- Opening a lazy-loaded screen produces a truthful loading state only if loading actually occurs.
- Cancel, confirmation and security controls are never deferred behind a nonessential lazy load.
- Measure startup time, memory and route load performance on target hardware.

---

## 10. Frontend architecture and state ownership

### 10.1 Stack

- Native application host: Rust application core with a narrow C++ adapter for CEF's native APIs.
- UI: React and TypeScript bundled with Vite and served through a registered first-party CEF application scheme.
- Browser engine: pinned Chromium Embedded Framework (CEF) release, using CEF Views/BrowserView-style native composition.
- Trusted control: Rust policy/orchestrator and CEF native bridge; no production CDP listener.
- Local inference: llama.cpp, using a tested platform backend and a managed private IPC channel.
- Persistence: local app-owned directories and platform secure storage for secrets.

The C++ layer is an adapter, not a second application backend. It handles CEF initialization, native view/window lifecycle, CEF-thread dispatch, browser lifecycle callbacks, renderer messaging and platform-specific host integration. It must not classify actions, authorize tasks, store credential values, or make product policy decisions.

CEF UI resources are served from a dedicated local scheme with a stable, explicit origin. App UI browser contexts and user website contexts must have separate storage/request contexts. Remote page content is untrusted and is not granted the app's native bridge.

CEF is chosen because it is expressly intended for embedded-browser controls and native application shells. The original Tauri/Wry plus separate Chromium composition is superseded by ADR-010 because it creates two independent rendering/input surfaces to synchronize. Keep React, TypeScript and Rust; replace only the desktop/browser hosting architecture.

### 10.2 Component organization

Organize around browser concepts. Adapt to the repository's actual structure and reuse verified components; do not create parallel shells that compete for authority.

Suggested conceptual tree:

```text
BrowserAgentApp
├── PlatformWindowFrame
├── TabStrip
├── BrowserToolbar
│   ├── BackControl (text label)
│   ├── ForwardControl (text label)
│   ├── UndoControl (permitted icon)
│   ├── RedoControl (permitted icon)
│   ├── ReloadControl (permitted icon)
│   ├── Omnibox
│   ├── BookmarksControl (permitted icon)
│   ├── DownloadsControl (permitted icon)
│   └── OverflowSettingsControl (permitted three-dot icon)
├── MainWorkspace
│   ├── ManagedChromiumSurface
│   └── PersistentAgentPanel
│       ├── TaskComposer
│       ├── AgentReadinessAndTaskState
│       ├── TaskActivityList
│       ├── InterventionAndConfirmation
│       └── VerifiedResult
└── SecondaryViews
    ├── History
    ├── Bookmarks
    ├── Downloads
    ├── Settings
    ├── PrivacyAndSecurity
    ├── CredentialsVault
    ├── ModelStatus
    ├── NetworkPolicy
    ├── Quarantine
    └── AuditDiagnostics
```

This is a logical model; use existing equivalent modules where they are sound. Do not implement all views as separate dashboards in the main browsing workspace.

### 10.3 Single ownership

- Browser URL, current tab and navigation state belong to the browser runtime.
- Target/frame/epoch identity belongs to the browser-perception/execution layer.
- Task lifecycle and orchestrator progress belong to the Agent runtime/orchestrator.
- Authorization and action classification belong to the Rust policy layer.
- Result confidence/outcome belongs to the Verification engine.
- Secret references and secret values belong to the vault/secure-fill boundary.
- React renders state and emits user intent; it must not manufacture authority or duplicate independent state machines.

### 10.4 Runtime state contract

All user-visible status is mapped from real runtime events. No frontend timers or canned response text may pretend that inference, navigation, downloads, policy checks or verification are happening.

The app must expose distinct browser and Agent state machines. A page still loading is not the same as the Agent thinking. An Agent waiting for confirmation is not the same as the browser being idle.

---

## 11. Managed CEF browser runtime

### 11.1 Runtime invariants

[BA-CEF-001] Browser-Agent owns and manages the CEF runtime and its process lifecycle.
- Pin an official CEF binary distribution/release branch and record its Chromium version, CEF version, platform targets, checksums, license notices and security-update policy.
- Use CEF native APIs for production navigation, tab lifecycle, frame lifecycle, browser commands, input forwarding, resize, focus and close operations.
- Do not enable a production CDP endpoint. CDP is restricted to isolated development/test builds and disposable test profiles.
- Use a separate app-owned profile/data root. Never attach to, reuse or migrate directly from the user's ordinary Chrome profile.
- Preserve Chromium sandboxing and site isolation. Never use `--no-sandbox`, `--disable-web-security` or `--disable-site-isolation-trials`.
- Do not silently fall back to stock Chrome, an unmanaged external browser, a screenshot, a platform WebView or a floating browser window.
- The page shown to the user must be the actual managed CEF browser view that receives mouse, keyboard and Agent input.
- Use CEF's normal native/Views rendering path for the interactive webpage. Off-screen rendering may be considered only if profiling proves it meets responsiveness, compositing and input requirements; it is not allowed as a shortcut to bypass correct native layout.
- Never expose a generic privileged JavaScript bridge to remote website frames.

### 11.2 Native composition and multi-window contract

[BA-BROWSER-001] Each Browser-Agent application window is one native top-level window with a deterministic CEF native view hierarchy. The product supports multiple such windows; “one window” means one integrated native composition per application window, not a single-window application. Within each window, the tab strip, 44 px navigation toolbar, managed website view and permanent Agent panel occupy deterministic non-overlapping view bounds. React renders app-owned chrome and the Agent panel in the fixed local UI context; each remote website is rendered by a separate CEF browser view in the appropriate profile/request context.

The native host owns view bounds, z-order, focus routing, resize events and CEF thread dispatch. React reports desired layout state only; it does not manipulate native handles or assign authority. Input over the website region must reach the active user webpage, not an invisible shell overlay. Input over app-owned controls must reach the app shell. The webpage must not be covered by a transparent app WebView that swallows pointer or keyboard events.

A feasibility proof must demonstrate the same composition on macOS Apple silicon and Windows x64, including the packaged application and multiple windows. Verify mouse, trackpad, keyboard, text selection, IME/input methods, browser context menus, focus transfer, scrolling, window resize, full-screen, minimize/restore, closing the active window, multiple monitors and high-DPI scaling. Closing a window with the active task must follow Section 2.14's safe-stop contract. Browser view and shell geometry must remain aligned under repeated resize/display-scale changes.

### 11.3 CEF lifecycle and frame coverage

Implement and test:

- CEF initialization, browser subprocess startup, orderly shutdown and cleanup;
- renderer/GPU process crash, browser close, unexpected process exit and safe restart policy;
- browser views, tabs, active-tab identity, background tabs and tab destruction;
- main-frame and subframe navigation, frame identity and renderer-context recreation;
- cross-origin frames and out-of-process iframes as supported by the selected CEF release;
- shadow DOM and accessibility-tree observation for supported workflows;
- navigation/network/lifecycle events and relevant DOM mutation/stabilization signals;
- active-tab focus, viewport, scroll position, scale factor and hit-target geometry;
- downloads, final file existence and metadata;
- stale frame IDs, node handles, target IDs and renderer contexts;
- session/profile isolation between local app UI and remote browsing tabs;
- native file-dialog, permission and external-protocol lifecycle.

Cross-origin iframe support is a release-blocking behavior, not a feature assumed to work because CEF supports Chromium. A test fixture must include same-origin, cross-origin, nested, replaced and detached frames. If the selected CEF build cannot establish safe frame identity for an action, the Agent returns `UNKNOWN` or asks for user intervention rather than acting through a guessed frame.

### 11.4 Observation and execution bridge

The production observation channel is a narrow, versioned CEF renderer-process-to-browser-process message contract implemented through CEF native process messaging, not a page-visible JavaScript object. It reports page-controlled data—DOM/accessibility summaries, frame identity, geometry and observation epoch—as **untrusted observations**. The observer may collect bounded structured data; it may not authorize actions, access secrets, call arbitrary filesystem/shell commands or issue privileged Browser-Agent commands. Any observation derived from webpage DOM, ARIA, labels or script-exposed state remains untrusted even when transported through a native CEF callback.

- The privileged React/native message bridge is registered only for the exact `browser-agent://ui/` origin and intended app UI frames.
- Remote website frames receive no app-control bridge. Page messages cannot impersonate trusted user confirmations or runtime policy events.
- Renderer messages are schema-validated in the browser process and then passed to Rust as untrusted input.
- All action decisions pass through Rust policy. The CEF native adapter performs only the already-authorized typed operation.
- Browser-originated page observations are never treated as proof of authorization. Website text, ARIA labels, DOM attributes and JavaScript messages are page-controlled and untrusted.
- Every wait is evidence-based, time-bounded, cancellable and tied to a specific browser/frame observation.

### 11.5 CEF debugging and test harness

Production release builds must have remote debugging disabled and must not publish a CDP listener, DevTools discovery endpoint, local HTTP control server or equivalent unauthenticated browser-control interface.

A dedicated development/test build may enable CDP only for disposable test profiles, on a test-controlled local interface, for the shortest necessary duration. The test harness must terminate the browser and remove the profile after the run. Production binary inspection must confirm the debugging endpoint is disabled. Test builds must not be used for ordinary browsing with real credentials.

Automated tests should primarily use a test-only adapter that invokes the same CEF-native operations and Rust policy boundaries used by production. Do not make production security depend on `chromiumoxide` support for draft pipe changes. Maintain a small protocol-conformance suite using CDP only where it adds coverage, not as the production privilege channel.

### 11.6 Browser profile, distribution and permissions

- Keep the browsing profile under an application-owned data directory with user-only permissions where the platform supports them. Keep the local app UI on its isolated `browser-agent://ui/` origin with no persistent website cookie store.
- Never use the default Chrome data directory, import credentials by copying Chrome profile files, or attach to the user's installed Chrome process.
- Browser-profile import, if added later, must be a separate explicit feature with a preview, category-by-category selection, integrity/error handling and no direct transfer of raw credential stores. It is not part of the baseline product.
- Use separate storage contexts for app-owned UI and remote website tabs.
- Define recovery for a corrupted profile without silently deleting user data. Offer a safe backup/reset path.
- Package CEF resources, locales and required subprocess executables with the correct per-platform app structure. Verify macOS signing/notarization and Windows installer behavior from clean machines.
- Track CEF/Chromium security updates, run compatibility tests before an update, and block release if the update weakens sandboxing, site isolation, context separation or Agent controls.
- Site permissions, downloads, external protocol links and file dialogs follow Section 17's permission model.

### 11.7 First page and manual browsing

On first launch, open the first-party local new-tab page. Manual browsing must not depend on model health. If CEF or a required security prerequisite fails, show a truthful actionable error and do not switch to an unmanaged browser silently.

## 12. Agent engines and task lifecycle

### 12.1 Five-engine architecture

1. **Perception:** constructs a grounded view of the current page and its actionable elements.
2. **Reasoning:** interprets the task and proposes a typed next action using bounded context.
3. **Policy:** authorizes, refuses or requests confirmation for the proposed action.
4. **Execution:** performs only the policy-authorized action through the managed browser.
5. **Verification:** independently evaluates whether the intended result occurred.

Supporting trusted components include Task Normalizer, Permission Check, orchestration/task authority, secure-fill boundary, skill/macro store, audit log and resource governor. These are not extra reasoning authorities.

### 12.2 Task lifecycle and transition contract

The task state machine is owned by the Rust orchestrator. Use these task states exactly unless a reviewed schema migration adds a state without changing existing semantics:

`CREATED`, `NORMALIZING`, `OBSERVING`, `PROPOSING`, `POLICY_CHECK`, `EXECUTING`, `VERIFYING`, `WAITING_FOR_USER`, `USER_TAKEOVER`, `PAUSED`, `BLOCKED`, `FAILED`, `CANCELLED`, `COMPLETED`.

Verification outcome is a separate field and uses Section 15.5's five-value vocabulary. `UNKNOWN` is an outcome, not a task-lifecycle state. A task can terminate as `COMPLETED` with outcome `UNKNOWN` when its work stopped normally but the result could not be proven. [BA-STATE-002] Whenever the task is `WAITING_FOR_USER`, the event must contain exactly one typed `wait_reason`: `ACTION_APPROVAL`, `USER_INPUT`, `AUTHENTICATION` or `TASK_BUDGET`. The primary copy and available controls come from that reason as shown in Sections 2.9.1 and 2.9.2. Missing or unknown reasons are schema-invalid and must fail closed rather than render a generic “Continue” action.

Legal transitions:

| Current state | Allowed next state(s) |
|---|---|
| `CREATED` | `NORMALIZING`, `CANCELLED`, `FAILED` |
| `NORMALIZING` | `OBSERVING`, `WAITING_FOR_USER`, `CANCELLED`, `FAILED` |
| `OBSERVING` | `PROPOSING`, `WAITING_FOR_USER`, `PAUSED`, `USER_TAKEOVER`, `CANCELLED`, `FAILED` |
| `PROPOSING` | `POLICY_CHECK`, `OBSERVING` (bounded retry after invalid proposal), `WAITING_FOR_USER`, `PAUSED`, `USER_TAKEOVER`, `CANCELLED`, `FAILED` |
| `POLICY_CHECK` | `EXECUTING`, `OBSERVING` (authorization invalidated; reobserve), `WAITING_FOR_USER`, `BLOCKED`, `PAUSED`, `USER_TAKEOVER`, `CANCELLED`, `FAILED` |
| `EXECUTING` | `VERIFYING`, `USER_TAKEOVER`, `FAILED` |
| `VERIFYING` | `OBSERVING`, `WAITING_FOR_USER`, `COMPLETED`, `FAILED`, `CANCELLED` |
| `WAITING_FOR_USER` | `POLICY_CHECK` (approved bounded action after fresh observation), `OBSERVING` (user answers/asks to continue or budget extension is approved), `USER_TAKEOVER`, `PAUSED`, `CANCELLED`, `FAILED` |
| `USER_TAKEOVER` | `OBSERVING` only after explicit Resume and fresh perception, or `CANCELLED`, `COMPLETED` |
| `PAUSED` | `OBSERVING` only after explicit Resume and fresh perception, or `CANCELLED`, `COMPLETED` |
| `BLOCKED`, `FAILED`, `CANCELLED`, `COMPLETED` | Terminal for that task; start a new task instead of mutating a terminal task |

Runtime invariants:

- Every transition is emitted by Rust with a monotonically increasing task revision and event sequence. React renders the authoritative state and cannot synthesize state changes.
[BA-WIN-002] There is at most one active autonomous task across the entire Browser-Agent application process, including all normal windows. It is bound to one owning window and one tab at a time; it may work across tabs sequentially only after fresh perception and policy checks. No other task may be queued or execute silently. This process-wide limit is stricter than a per-window limit and is chosen for the 8 GB reference device, single model-runner lifecycle and cross-window authorization clarity; it must remain consistent with Section 2.14.
- Each action follows `OBSERVING → PROPOSING → POLICY_CHECK → EXECUTING → VERIFYING` where relevant. Deterministic observations may skip reasoning only through an explicitly defined, tested skill path that still passes policy.
- Pause and cancellation requests are acknowledged immediately but do not pretend an already-dispatched native event can be undone. If a write may already have reached the page, verify/reconcile its result before declaring the task cancelled or safe to resume.
- User input in a page or a change of active tab while automation owns control immediately blocks dispatch of new actions, invalidates the observation epoch and enters `USER_TAKEOVER` or `PAUSED` according to the source of the interruption.
- A return from `USER_TAKEOVER` or `PAUSED` requires explicit user-initiated Resume, browser health validation, fresh perception, target reconciliation and policy reauthorization. Never replay the last action solely because its completion event is absent.
- `WAITING_FOR_USER` must identify whether it requests approval for one bounded reversible action or hands over actual browser control. These cases cannot share an ambiguous generic “continue” button.
- Ignore duplicate/stale events using task ID, task revision and monotonic sequence. Invalid state transitions are logged and fail closed.
- Deadlines, cancellation tokens and loop limits apply to every blocking/waiting state.
- [BA-NORMALIZE-001] The normalizer may ask at most **two rounds of clarifying questions** per task. If ambiguity remains after two rounds, state the assumptions explicitly and proceed only with bounded READ actions or proven REVERSIBLE_WRITE actions that do not depend on the unresolved detail. If the ambiguity changes the destination, secret disclosure, external side effect, irreversible effect or authorization scope, do not guess; stay in `WAITING_FOR_USER` with `USER_INPUT` until the user answers or cancels.
- [BA-TABSCOPE-001] A task is bound to its originating window and owns the originating tab plus task-created tabs permitted by popup/navigation policy. The model may not act in other tabs or windows merely because they exist. A user can explicitly grant an additional already-open tab; granting it adds its origin/tab identity to the task scope and is audited. Tab tear-off of task-owned tabs is blocked until the task is paused/cancelled/taken over and authorizations invalidated.
- [BA-STUCK-001] If two consecutive dispatched actions with the same normalized action/target digest produce no expected semantic state change and no new admissible evidence, do not dispatch a third identical action. Enter `PAUSED` with user-facing copy `The agent appears stuck`; offer `Retry observation`, `Take over` and `Cancel task`. Repeating a rejected proposal is not progress. A WRITE is never retried automatically under this rule.

### 12.3 Typed action grammar

The model must produce a schema-constrained typed action, not arbitrary code or uncontrolled browser commands. The initial action vocabulary includes:

- `CLICK`
- `TYPE`
- `SELECT`
- `NAVIGATE`
- `SCROLL`
- `WAIT`
- `SECURE_FILL`
- `REQUEST_CONFIRMATION`
- `HANDOFF_TO_USER`

Each action must have a closed schema with required fields, bounded strings/arrays/numbers, allowed enums and explicit refusal behavior. The model cannot emit arbitrary JavaScript, shell commands, native API calls, CDP commands, filesystem paths for unrestricted access or generic tool invocations. Unknown action types, extra authority-bearing fields and malformed payloads are rejected. Model output is untrusted input.

`REQUEST_CONFIRMATION` is a structured request for approval of one specified action that policy already classifies as eligible for agent execution (normally a bounded `REVERSIBLE_WRITE`). Approval is single-use and expires when the target, origin, parameters or relevant page state changes. It does not override the action-class policy.

`HANDOFF_TO_USER` stops agent dispatch, sets the control owner to the user, and places the real CEF browser on the target page for a consequential or ambiguous action. It never clicks, types, submits or otherwise executes the handed-off action. After the user finishes, the Agent remains stopped until explicit Resume; new perception and policy checks are required.

### 12.4 Target-grounding fallback chain

Resolve action targets through a deterministic evidence hierarchy, such as:

1. Semantic target identified from current perceptual state graph.
2. Accessible role/name and accessible tree.
3. Stable, grounded attributes.
4. DOM relationships and local semantic context.
5. Narrow CSS selector, if previously grounded and still valid.
6. Bounded LLM recovery proposal, followed by complete target and policy validation.

Never let the model invent a selector and execute it without current-page grounding. Stale epoch/node references must be rejected. Re-perceive after significant DOM/navigation changes.

### 12.5 Perception rules

- Build a semantic state graph with stable identifiers scoped to a specific observation epoch.
- Include actionable elements, roles, accessible names, visible text, relevant attributes, geometry, frame origin and relationship context as appropriate.
- Exclude scripts, comments, hidden content, zero-sized elements, off-screen content where irrelevant and `aria-hidden` content where it is not exposed to the user.
- Mark all website text, attributes, accessible names, metadata and tool-returned page content as untrusted data. They may inform task interpretation but cannot override user intent or policy.
- Mask passwords, OTPs, credentials, payment secrets, auth headers, cookies, session tokens and other sensitive values before normal frontend/model exposure.
- Do not include cookies, tokens or auth headers in logs, prompts, screenshots or task history.
- Distinguish observed labels from inference. If accessible name and visible text materially conflict, classify the target/action as `UNKNOWN` and seek fresh evidence or user input.

[BA-OBSERVATION-001] Serialize observations to the model as stable-order structured data, never raw HTML or script source. Order evidence deterministically: current target and its local accessible/DOM relationships first; visible actionable elements next; relevant nearby text and page metadata afterward. Per observation, cap the semantic graph at 2,000 nodes, 256 actionable candidates, 12 relationship levels and 12,000 serialized model-input tokens. Apply a 750 ms collection budget excluding explicit navigation/stabilization waits. When a limit is reached, emit a typed truncation summary with counts by omitted role/class; never silently omit data that changes action safety. The model may receive task text, current origin/title, visible relevant text, roles/names/attributes that are safe, geometry and bounded action history; it never receives arbitrary page HTML, scripts, raw network bodies, cookies, auth headers or vault values. Sensitive field verification occurs inside trusted code and returns a boolean/status/pattern result rather than the underlying value. Explicit observation invalidation triggers are: top-level navigation or redirect; origin/scheme/port change; active-tab/window change; frame or renderer-context creation/destruction/replacement; browser crash/reconnect; user input/takeover; relevant DOM/accessibility mutation; scroll/viewport/zoom/device-scale change that can change target geometry; popup/new target creation; permission/certificate interstitial; download transition that changes current page behavior; and any hit-test mismatch. Prefer incremental re-observation from supported CEF mutation/accessibility signals, but force a fresh full bounded observation when identity or origin changes. An incremental update must never preserve authorization for a target whose preconditions changed.

[BA-RETRY-001] Retry policy is action-class-specific. `WAIT` and observation stabilization may retry at most twice within their deadline with bounded backoff. A `READ`/safe navigation may be retried at most once only after proving the previous dispatch did not occur or reconciling its result, then re-observing and reauthorizing. `CLICK`, `TYPE`, `SELECT`, `SECURE_FILL`, download initiation and any `WRITE` are never blindly retried. If dispatch status is ambiguous, reconcile and report `UNKNOWN` if evidence remains insufficient. Retries apply to observation/proposal readiness, not to repeating a potentially executed side effect.

### 12.6 Stale action and click correctness

- Validate the current observation epoch immediately before execution.
- Before action, resolve the grounded node/frame identity and verify its current geometry in a fresh observation. Recheck visibility, enabled state, bounding box, frame identity and observation epoch immediately before dispatch.
- Dispatch pointer/keyboard input through the active CEF browser host so Chromium performs actual native input targeting. Do not authorize a click based solely on a model-invented coordinate or stale image.
- Where hit testing is available in the active renderer context, compare the intended target with the current topmost element at the click point. Treat the result as untrusted page observation, not as a security authorization.
- Abort and re-perceive when the target moved, detached, became obscured, changed identity, changed frame, or no longer meets the preconditions.
- Any navigation, relevant semantic epoch change or renderer-context replacement invalidates dependent unresolved actions and pending confirmations.
- Never click based only on a coordinate that originated from a stale screenshot or old DOM snapshot.

### 12.7 Independent verification

Verification must use evidence distinct from the model's assertion that it succeeded. Depending on the task, evidence may include:

- actual URL/navigation state;
- a changed DOM or accessible state;
- a confirmation element or record;
- download/file completion metadata;
- a server-visible result page;
- a task-specific postcondition;
- stable before/after values.

Do not claim success merely because a click was issued, a button animation occurred or a tool returned without error. If evidence is insufficient, return `UNKNOWN`/unverified and explain what is missing.

---

## 13. Policy authority, actions and human gates

### 13.1 Authority boundary

[BA-POLICY-001] The model proposes; it never authorizes.
- Rust policy evaluates every action before execution.
- The executor can perform only an action accompanied by a valid authorization bound to its action, target, context and current observation.
- Authorization cannot be reused across unrelated targets, origins, epochs or action parameters.
- User confirmation is an explicit policy transition, not a button that merely changes the UI.

### 13.2 Action classes

Use the following class meanings:

- **READ:** inspect page state or perform a user-intended navigation/data retrieval without intended mutation of user/account/external state. A READ request can still disclose ordinary network metadata to the destination; this disclosure must be reflected in user-visible privacy copy.
- **REVERSIBLE_WRITE:** a bounded local or website change for which a tested compensating operation or discard path is evidenced to restore the relevant prior state without meaningful residual external consequence. [BA-REV-001] Before using this class, policy must have a capability record naming the exact operation, target type, required preconditions, stable resource identity, expected postcondition, compensation/discard operation, compensation postcondition, known residual effects, expiry boundary and test evidence from the supported workflow. Before compensation, re-observe and confirm the expected post-state still holds; compensation is a new single-use authorized action and must be independently verified. A UI label (“draft”, “edit”, “undo”), an assumed website convention, or the model's assertion is not evidence. If a notification, public visibility, message, remote workflow trigger, permission grant or other external side effect may remain, classify the original action as `IRREVERSIBLE` or `UNKNOWN`, even when the visible UI can be changed back.
- **IRREVERSIBLE:** payment, purchase/order submission, destructive action, publication, message send, permission grant, credential/secret disclosure, or any action with a consequential or non-recoverable external side effect.
- **UNKNOWN:** effect, target, destination, reversibility, permission or authority cannot be established from trustworthy runtime evidence.

`UNKNOWN` is handled at least as strictly as `IRREVERSIBLE`. Never downgrade an unknown action because the model says it is safe. An action class is a runtime policy result, not a label chosen by the model.

### 13.3 Required policy checks

- Bind every authorization to a unique action ID, action type, validated parameters, semantic target, browser/tab identity, frame identity, exact origin, current observation epoch and an expiry/consumption state.
- An authorization is single-use. The executor atomically consumes it before dispatch and rejects replay, expiry, parameter changes, target changes or context changes.
- Revalidate origin, frame, field and target immediately before credential use or any consequential write.
- Treat navigation, redirects, deep links, URL parameters, downloads and form destinations as potential exfiltration/side-effect channels. Navigation is not automatically safe merely because its action type is `NAVIGATE`.
- [BA-EGRESS-001] Treat every Agent-caused typed value, selected option, form submit, URL parameter, downloaded request and external-protocol launch as an egress decision. The policy layer classifies the data's provenance/sensitivity, exact destination origin, whether the user asked to disclose it, and whether the value is credential/vault-derived. Values marked origin-bound (credentials, OTPs and other secrets or user-defined origin-restricted data) may go only to their exact authorized origin. For other task/user-derived data, determine whether the task clearly authorizes sending it to the chosen destination and request approval when sensitivity or intent is ambiguous. Block egress if an origin is unauthorized, a secret is used without an approved secure-fill reference, the destination changed, or the page is attempting to redirect/reframe the user's intent. The model never decides egress alone. Known vault secrets are compared inside the trusted vault/policy boundary using safe fingerprints where feasible; raw vault values must not be copied into logs or model prompts for detection. When a value is sensitive or egress intent is ambiguous, ask the user or refuse.
- [BA-ALLOWLIST-001] The optional `Only use sites I've already opened` constraint in Section 2.9.2 is enforced in Rust against the set of origins captured/approved for the task. Turning it off does not disable egress checks; turning it on does not claim to sandbox the websites' own subresource network traffic.
- [BA-CLIPBOARD-001] The Agent never reads from or writes to the operating-system clipboard by default. A task that requires clipboard access must request one explicit, scoped, single-use permission describing read versus write and the data purpose; permission expires after that operation and is audited. Remote website clipboard APIs remain denied until a user grants the corresponding per-origin browser permission. Ordinary manual copy/paste by the user remains available.
- [BA-UNMASK-001] Revealing a value deliberately masked in Browser-Agent-owned UI (for example, a vault entry or a redacted task artifact) requires a conscious user action and writes an audit event containing which record/category was revealed, who initiated it, time and context—but never the revealed value. The action is unavailable to the Agent and to website content.
- [BA-GRANT-001] The Agent may never grant or broaden site/device permissions. Enabling a high-risk origin policy requires the user to type the canonical hostname being granted. The Agent cannot bypass certificate interstitials. If manual bypass is supported at all, require a typed exact hostname and clear certificate-error details; record the user's action and prevent Agent execution on that origin for the remainder of the task unless a separate explicit policy rule allows a safe read-only path.
- Classify effects based on the actual target, current page state, parameters, destination, reversibility and external side effects—not on the verb or page label alone.
- Treat page-controlled labels, instructions, ARIA names, DOM attributes, confirmation-like UI, and action descriptions as untrusted data. A page cannot grant permission or waive policy.
- A second trusted check is required when the action's apparent safety or classification depends on page-controlled information.
- The user task remains the authority if page content attempts to reframe the user's request. If intent is ambiguous, stop and ask; do not infer broader permission.
- User confirmation is bound to the exact structured action, destination, values, and consequences presented. Any meaningful change invalidates the confirmation.
- Confirmation content is rendered from structured trusted runtime fields, never exclusively from model prose.
- Reject unknown action types, unrecognized parameters, malformed schemas, stale references, unexpected destinations and any action for which the effect cannot be bounded.
- Policy must be fail-closed for sensitive actions when the policy engine, audit persistence, profile identity or origin/frame validation is unavailable.

[BA-POLICY-PERF-001] The pure Rust policy checks must not invoke the model. On reference hardware, target p95 ≤10 ms for deterministic action/authorization checks and ≤50 ms for checks that include origin/egress/capability-record lookups under the preregistered fixture. Instrument policy latency locally, never upload it. If the measured p95 exceeds the limit, optimize or simplify the bounded policy computation without removing checks; do not bypass the policy or cache a verdict across epochs. Any cache key must include the complete authorization context and be invalidated on relevant state changes.

### 13.4 Consequential tasks and checkout

- The Agent must never execute an `IRREVERSIBLE` or `UNKNOWN` action. This includes payment, purchase/order placement, message send, publication, account deletion, destructive submission, external permission grant, or disclosure of a secret to an unapproved destination.
- The Agent may research, compare, select items, fill non-secret fields, draft content and prepare a cart/draft when those actions are independently authorized and can be safely interrupted.
- Before the final irreversible/unknown step, use `HANDOFF_TO_USER`, focus the real browser and show the precise action, destination/origin, material parameters and known consequences. The user performs the action manually.
- A UI confirmation button does not authorize the Agent to execute a prohibited irreversible action. It can acknowledge the handoff or approve an eligible bounded reversible action only.
- Following user takeover, do not resume automatically. The user must explicitly resume or finish the task; the Agent then re-perceives the current page and reports only what the evidence establishes.
- A user-requested consequential action does not remove the handoff requirement. User intent establishes the goal, but the user remains the executor of the final consequential operation.
- Do not claim that external checkout, message delivery, publication or account mutation succeeded unless the subsequent actual result is independently observed. If evidence is incomplete, report `UNKNOWN`.
- `READ` navigations to an explicit URL/search result may be automatic, but the policy must inspect destination origin, credentials in the URL, redirect behavior and whether query/path parameters disclose sensitive task data. Never put credentials or unnecessary personal data in a URL.
- A change in the intended destination after user approval, including a redirect to an unrelated origin, invalidates any pending action authorization. Do not auto-follow a redirect when doing so would disclose a secret or execute a new side effect without policy review.

### 13.5 Credential isolation and secure fill

- Secret values live only in the trusted Rust credential boundary and platform secure storage. The model receives opaque references or field-category metadata, never secret values.
- Secure fill is executed through a trusted, narrowly scoped path bound to the exact approved origin, browser tab, frame and field context. The native adapter may carry the secret only for the duration of the approved fill and must clear owned temporary buffers where feasible.
- The user must have explicitly enabled credential use for the exact origin or explicitly requested sign-in for that stored credential at that origin. No page instruction can trigger credential use by itself.
- Recheck origin, frame identity, target identity, focus and task authorization immediately before fill. Abort on redirects, cross-origin frame changes, target replacement, ambiguity or unexpected navigation.
- Exact origin means scheme, hostname and port. Credentials do not automatically inherit permission to a subdomain, sibling domain, alternate port or insecure HTTP origin. Additional origins require separate explicit user authorization.
- Cross-origin iframe fields are not eligible for fill unless that exact frame origin is separately authorized and the CEF runtime can prove the current frame identity.
- Secrets must not be included in React state, ordinary event logs, DOM snapshots, screenshots, model prompts, task summaries, crash reports or copied diagnostics.
- A CSS mask is not protection. Redact before data reaches general-purpose presentation code.
- A secret inserted into a webpage can be read by scripts or infrastructure controlled by that website. Browser-Agent protects the secret within its own trusted components; it cannot guarantee secrecy after sending the value to the approved destination.
- Do not automatically send secrets into a website merely because its content requests them. If a site requests a new secret category or a different credential, require a new user decision.
- TOTP/OTP is secret material and follows the same origin boundary, redaction and expiry rules. Do not put OTP codes in task history or audit metadata.
- Successful fill is not proof of successful authentication. Verify the resulting authenticated state independently without recording the secret or relying solely on a page-provided success message.

### 13.6 Manual password-manager and autofill behavior

[BA-PASS-001] Ordinary manual browsing includes a built-in local credential vault and explicit origin-bound credential fill; the user does not need to invoke the Agent to use saved logins. The vault is separate from the Agent reasoning process and uses macOS Keychain or the supported Windows platform-protected storage mechanism for secret values.

- Do not save or replace a password silently. Offer a clear “Save login?” prompt only after an explicit user-driven login submission and a credible success transition; user confirmation is required before storing or replacing a credential.
- Do not autofill on page load. Present a user-triggered fill affordance in app-owned browser chrome or a controlled native prompt. Confirm the exact `https` origin, port, frame and field identity before fill; do not use substring or registrable-domain-only matching as the authorization check.
- Never reveal credential values to page JavaScript as an app bridge result, the React UI, model prompt, ordinary logs or telemetry. The secret necessarily becomes available to website-controlled code after insertion into a webpage field; disclose this boundary in vault/help copy.
- Never autofill into HTTP origins, ambiguous fields, cross-origin frames without separate exact-origin authorization, hidden fields or a page whose origin/frame identity changed during the operation.
- Do not fill credentials in Private Windows, where the vault is disabled for browsing forms. Password-save prompts and credential suggestions are also disabled there.
- Password-manager operations for manual browsing are explicit user actions, not Agent `READ` or `REVERSIBLE_WRITE` permissions. Saving, replacing, exporting or deleting vault entries requires its own authenticated, auditable policy path.
- Do not implement silent password capture from form inputs or infer that a typed value is a password to be saved. Offer saving only in response to a user-initiated flow and then request explicit approval.

## 14. Privacy, network behavior and local storage

### 14.1 Local-first promise

[BA-LOCAL-001] Browser-Agent must not require vendor-cloud inference or an API key. Browsing context, page perception, model prompts, credential values, task history and audit evidence remain local by default.

This promise must be stated precisely: ordinary browsing necessarily sends user-initiated or browser-required requests to the websites the user visits. The privacy commitment is that Browser-Agent does not secretly relay browser content or secrets to a Browser-Agent cloud service, third-party analytics system or remote inference endpoint.

### 14.2 Network policy

- No background upload of page content, browsing history, credentials, task transcripts, screenshots or model context.
[BA-TELE-001] The production release has **no first-party remote analytics, usage telemetry, crash-report upload or background browsing-data upload**, and therefore no telemetry opt-in toggle is present or implied. The product sends no Browser-Agent telemetry by default. Ordinary website requests, user-requested navigation/search, explicit model/runtime downloads and user-initiated update checks are separate network actions and must be disclosed accurately.
- Local diagnostic records are stored locally with secrets and page content redacted. A diagnostic bundle can be created only by an explicit user action; preview the contents and redactions before writing/exporting it. Do not auto-upload it.
- Any future remote telemetry must be introduced through a new privacy ADR, a clear opt-in that is off by default, a visible data inventory/retention disclosure, minimal event schemas, a withdrawal path, and security/legal review. It is not allowed merely because a backend endpoint exists.
- Network use for the user's navigation is distinct from Browser-Agent service telemetry and must not be falsely described as offline browsing.
- Update/model/package downloads must be explicit, verifiable, integrity-checked and documented; no silent remote inference fallback.
- The production application must not expose an unauthenticated local HTTP/WebSocket control server, CDP endpoint, model HTTP server or privileged localhost API. The model runner uses managed stdin/stdout or OS-scoped IPC. Any future network listener requires a security review and a new architecture decision.
- The UI must accurately describe offline/blocked/degraded networking when relevant.
- Treat navigation and deep links as possible exfiltration routes. Apply policy to redirects, external origins and form destinations.

### 14.3 Local data and secrets

- Store ordinary settings and non-secret state in app-local storage appropriate to the repository architecture.
- Use platform secure storage for secrets where available (for example macOS Keychain and the supported Windows credential-protection mechanism), with a documented fallback/security posture.
- Keep browser profiles, histories, downloaded files, local model artifacts and audit information under explicit lifecycle rules.
- Provide user-visible controls for clearing relevant local data without misleading claims about data that remains in Chromium profiles or downloads.
- Prevent secrets from being written to crash reports, debug logs or diagnostic bundles.

[BA-DATA-001] Provide separate, truthful deletion actions: `Forget this site` (remove that origin's normal-profile cookies/site storage/cache/history/permissions and related saved research notes/task references, with a count/preview before confirmation), `Clear browsing data`, `Clear Agent task history`, `Delete research notes`, `Remove model files`, `Purge old audit records`, and `Clear all Browser-Agent data`. Credential-vault entries and downloaded files require separate explicit choices and are not removed by `Forget this site` or `Clear all Browser-Agent data` unless the user separately selects them. Clearing data does not imply secure erasure from SSD wear-leveling, OS backups or website servers; state the exact app-owned data removed.

[BA-VAULT-001] Vault secrets use macOS Keychain or Windows Credential Manager/DPAPI-backed storage where supported. Set the default vault auto-lock to 5 minutes of inactivity while the app is unlocked, and lock immediately on OS user-session lock, application quit or explicit user request. Unlock is user-mediated; repeated failed unlock attempts use the platform's trusted recovery mechanisms and do not create a custom biometric bypass. If platform secure storage is unavailable, vault creation is disabled rather than falling back to plaintext or a reversible app-only encryption key. Provide separate vault export/delete flows with explicit risk confirmation; never auto-export.

[BA-NOTES-001] Research notes are local-only and created only by explicit user action or explicit approval of a task's proposed notes. Each note stores a user-editable title/body, source URL/origin, capture time, observation-epoch reference and the specific excerpt the user chose to retain. Notes have exact-match local search, edit, export and deletion; they are never synchronized or added by private-window activity. Quotes/excerpts are evidence snapshots, not live-page truth; present the source link and capture date.



### 14.4 Privacy disclosures

Use concrete statements: what is stored, where it is stored, what websites receive, what network services are used, and how model assets are obtained. Do not use blanket “100% private” or “nothing ever leaves your device” copy if ordinary website traffic or explicit update downloads contradict it.

---

## 15. Audit, resume, resource limits and failure behavior

### 15.1 Audit records

[BA-AUDIT-001] Audit events must record sufficient non-secret evidence for debugging, safety review and task accountability, including task/action ID, action class, target/origin where safe, policy decision, execution result, verification state, CEF/browser context identity (non-secret pseudonymous identifiers), and timestamps.

- Canonicalize event bytes with a documented schema/version and deterministic encoding before computing integrity metadata.
- Use a chained HMAC construction in which each event MAC is bound to the previous MAC. Rotated segments carry the previous segment's final MAC and a segment identifier.
- Store the HMAC key in platform-protected storage and restrict access to the trusted runtime. Do not place the key in config files, source, environment dumps or ordinary logs.
- Document key creation, rotation, backup/recovery policy, integrity verification and response to key compromise.
- A local HMAC chain provides tamper evidence only while the key and trusted application boundary remain trustworthy. Do not describe it as proof against an attacker who can control the process and its key.
- Never include raw secrets, cookies, authorization headers, OTP values, model prompts containing sensitive data, or unredacted credential-bearing URLs.
- Persist required audit metadata before any operation that could cause an external side effect. If audit persistence fails, block further non-READ actions, transition the audit state to `DEGRADED`, and notify the user.
- Audit verification failures must be visible and must not be labelled “chain verified.” A broken chain is never silently reset; rotate only after preserving the failure evidence and recording the new segment's link status.

[BA-AUDIT-002] Use UTC timestamps in the audit log and a monotonic clock for elapsed durations/timeouts; convert to the user's locale only in the UI. Audit-write failure is a fail-closed safety event: stop new Agent dispatch, preserve the last safe state and show that required audit recording failed. Never silently drop a required event to continue consequential work. Rotate audit records in bounded daily or 10 MiB segments, whichever comes first, chained to the preceding segment MAC. Default audit/task-history retention is 90 days, configurable to 30/90/365 days or user-managed retention. Purging is explicit and irreversible; after pruning, report `Integrity verified for retained range` rather than claiming a complete chain over deleted segments. Private-window browsing/events never enter the persistent audit/task-history store; security events needed to record a private-mode policy refusal may contain only minimal mode/reason metadata, not page origin, title or private content.

### 15.2 Resource governance

[BA-RESOURCE-001] Bound model context, inference duration, browser waits, task retries, history buffers and page-observation size.

[BA-TASK-BUDGET-001] Default hard per-task budgets on the 8 GB reference profile are: 60 policy-evaluated action proposals; 50 dispatched actions; 20 Agent-initiated top-level navigations; 64,000 cumulative model input tokens; 8,000 cumulative model output tokens; and 15 minutes of active task execution time. User-wait and explicit paused intervals are excluded from active execution time but are shown as elapsed wall time. Only top-level Agent-initiated navigation/download actions count against navigation budget; do not pretend the app can count or limit all subresource traffic generated by a website. Budgets are defaults for the measured task envelope, not safety authorization. The user may approve one extension of up to 50% for a single exhausted budget, with exact new totals shown and audited. Hard memory ceilings, policy rules, privacy restrictions and no-autonomous-irreversible-action rules cannot be extended by the user.

[BA-MEMORY-001] Enforce hardware-class resource profiles. On the 8 GB Apple-silicon reference profile, model plus inference-runner resident memory is capped at 2.5 GB and total Browser-Agent processes at 6.0 GB; park inference before the total reaches the cap (at 5.5 GB combined app-process memory) or immediately on OS high-memory-pressure notification, whichever occurs first. On the Windows 16 GB reference profile, model plus runner is capped at 4.0 GB and total Browser-Agent processes at 9.0 GB; park inference at 8.5 GB or on sustained OS memory pressure. These are initial safety ceilings and must be validated on actual devices; never raise them only to improve a benchmark score. Do not restart/re-warm inference automatically in the middle of a task. Offer Resume only after memory headroom is restored and report measured re-warm latency; target p95 re-warm to readiness ≤10 seconds on the selected profile.

[BA-STARTUP-001] On the reference devices, target p95 time from process launch to first interactive native window ≤3 seconds and to an interactive address field ≤4 seconds with a warm OS filesystem cache; cold-start measurements must also be reported separately. CEF startup and local new-tab rendering are on the critical path; model scanning, integrity hashing, model loading, optional on-device benchmarks and secondary-view code are not. If a required browser integrity check prevents safe browsing, fail truthfully rather than bypassing it to meet the SLO.

Memory-profile breakdown under the standard reference workload (4 open tabs on the 8 GB profile; 8 open tabs on the 16 GB profile):

| Hardware profile | CEF browser/renderer/GPU/utility group | Model + inference runner | Rust/React/native UI and support processes | Total Browser-Agent process budget |
|---|---:|---:|---:|---:|
| 8 GB Apple silicon `COMPACT` | ≤3.0 GB | ≤2.5 GB | ≤0.5 GB | ≤6.0 GB |
| 16 GB Windows x64 `STANDARD` | ≤4.0 GB | ≤4.0 GB | ≤1.0 GB | ≤9.0 GB |

Measure per-process RSS and OS-reported physical-memory footprint separately; avoid double-counting shared pages in aggregate footprint where the OS exposes a reliable metric. If a component budget is exceeded even while total memory is below its cap, investigate rather than silently borrowing from another component's security/control budget. Parking inference must not terminate an in-flight browser action; it stops new model calls at a safe task boundary, then unloads the model. Add a user setting `Park model when OS Low Power Mode/Battery Saver is active`, enabled by default. Low battery alone does not interrupt a task; critical memory pressure always takes precedence.
- Apply configured timeouts and backpressure; do not allow infinite polling or unbounded DOM snapshots.
- Resource exhaustion must not cause a policy bypass or silent cloud fallback.
- On a hard resource stop: **STOP → preserve safe state → write audit event → notify the user → require intervention or an explicit safe resumption path.**
- Preserve the last known safe checkpoint and mark current results unknown when proof is incomplete.

### 15.3 Default loop/livelock protections

The following are conservative bootstrap limits, not permanently justified constants:

- At most 8 relevant semantic epoch changes in a rolling 2-second window before reevaluating/pausing for livelock.
- At most 3 failed stabilization windows in 10 seconds before stopping or requesting intervention.
- Delegate biometric attempt limits, lockout and recovery to the operating system's supported authentication API. Do not emulate, override or promise a universal biometric retry count. If an app-owned re-authentication throttle is needed, make it a separately reviewed policy with explicit threat rationale and tests.

[BA-LOOP-001] Calibrate livelock thresholds using a frozen, labeled suite of at least 1,000 benign dynamic-page runs and 300 adversarial/no-progress runs, separated into tuning and held-out sets by site/workflow family so near-duplicates do not leak. Include legitimate React/Vue-style updates, streaming pages, navigation churn, detached frames, repeated prompt injection, targets that never stabilize and delayed-but-valid page transitions. Report false-pause rate on benign workflows, detection rate and time-to-stop on seeded loops, critical unsafe-continuation rate, extra actions after the first detectable no-progress signal, and confidence intervals. The held-out release targets are: false-pause rate ≤5% on benign cases; detection of at least 99% of seeded no-progress/livelock cases before 20 Agent-dispatched actions or 5 seconds, whichever comes first; and zero critical unsafe continuations in adversarial cases. Recalibration requires a reviewed ADR citing the distribution shift, test artifact hashes and held-out result. Never tune on the release hold-out set. If the targets conflict, prefer stopping safely and narrow the supported workflow rather than silently weakening a security invariant.

The limits must be configurable through the reviewed policy layer and exercised in tests. Do not implement them only as frontend text.

### 15.4 Security evaluation and release gates

[BA-SEC-001] Security numbers are meaningful only with fixed definitions, denominators and test artifacts. Keep the following as provisional release targets, not claims about present implementation:

- **Critical policy bypass:** zero tolerated in deterministic invariant tests. A single reproduction that allows an unauthorized high-impact action blocks release regardless of aggregate scores.
- **Security-classification false negatives:** target at most 0.5% on the held-out unsafe-action corpus. The denominator is the number of ground-truth unsafe actions; it is not the total corpus size.
- **Minimum corpus:** at least 600 adjudicated unsafe/consequential examples and at least 600 adjudicated safe/eligible examples, plus a separate adversarial suite for prompt injection, malicious redirects, stale authorization, secret exfiltration, duplicate events, compromised UI messages, frame changes and crash ambiguity. Use independent scenarios rather than merely paraphrased copies.
- **False positives:** target at most 10% unnecessary denial or confirmation on the supported, low-risk eligible-action set. Report unnecessary confirmation separately from false denials.
- **Held-out methodology:** separate development/tuning data from a frozen held-out set; label action class, expected decision and severity; record provenance, adjudication disagreements, excluded cases, model/runtime/build versions, seeds and confidence intervals.
- **Statistical interpretation:** with zero misses among 600 independent unsafe trials, the approximate one-sided 95% upper bound is about 0.5% (rule-of-three approximation). The benchmark must show the observed count and interval, not just a percentage. Because adversarial examples are not necessarily independent or representative, this interval does not replace threat-based tests or deterministic invariants.
- **Policy/executor boundary tests:** reject malformed action payloads, replayed/expired authorization, origin/frame/epoch mismatch, unexpected parameter mutation, unauthorized local-scheme navigation and every attempt to call native privileges from website content.
- **Release decision:** critical invariant failures block release. Metrics that miss targets require remediation and a rerun of the frozen suite; do not tune against the frozen set and then reuse it as an unbiased final evaluation.

Metrics must be computed from executed test artifacts, not inferred from documentation or self-reported by a model.

### 15.5 Verification vocabulary and evidence rules

Use these outcomes consistently:

- `VERIFIED_SUCCESS`
- `LIKELY_SUCCESS`
- `UNKNOWN`
- `LIKELY_FAILURE`
- `VERIFIED_FAILURE`

The verifier must evaluate task-specific preconditions and postconditions using evidence obtained from the current browser/runtime state. Model statements, tool return codes, button animations and page text alone are not sufficient proof.

| Outcome | Required interpretation |
|---|---|
| `VERIFIED_SUCCESS` | All mandatory postconditions hold and are backed by fresh, task-appropriate evidence with no unresolved contradiction. |
| `LIKELY_SUCCESS` | Evidence strongly suggests success, but at least one noncritical required proof is unavailable. The UI must say the outcome is not fully verified. |
| `UNKNOWN` | Evidence is missing, stale, contradictory, interrupted, or cannot distinguish whether the side effect occurred. Never color or phrase this as success. |
| `LIKELY_FAILURE` | Evidence suggests the postcondition was not achieved, but the failure is not conclusively established. |
| `VERIFIED_FAILURE` | Fresh evidence establishes that the required postcondition failed or the action was rejected. |

Minimum task-specific evidence examples:

- **Navigation:** active tab URL/origin and current page state match the intended destination.
- **Form fill:** expected non-secret fields contain the intended values; secret values are not copied into verification records.
- **Download:** CEF reports completion, the file exists under the expected destination, and file metadata is plausible. Completion alone does not prove the file is safe.
- **Setting change:** read back the setting or independently observe the new state after the action.
- **Credential fill:** confirm only that the authorized fill operation occurred; authentication success requires separate evidence.
- **Task recovery:** reconcile current browser state with the checkpoint and confirm no ambiguous write is blindly replayed.

If sources of evidence conflict, choose `UNKNOWN` until reconciled. User-facing copy must clearly distinguish task termination from outcome confidence. Only `VERIFIED_SUCCESS` gets the verified-success color. [BA-VERIFY-001]

Task lifecycle, task termination and verification outcome are orthogonal. Apply this UI mapping independently of Section 2.9.1's primary lifecycle label:

| Verification outcome | User-facing outcome label | Meaning |
|---|---|---|
| `VERIFIED_SUCCESS` | `Verified success` | A task-specific postcondition is supported by sufficient independent evidence. |
| `LIKELY_SUCCESS` | `Likely succeeded — not fully verified` | Positive evidence exists, but a required proof is missing or incomplete. Never use success green. |
| `UNKNOWN` | `Result unknown` | The side effect may have happened or current evidence is insufficient/conflicting. Explain what is known and what evidence is missing. |
| `LIKELY_FAILURE` | `Likely did not succeed` | Evidence suggests the postcondition was not achieved, but is inconclusive. |
| `VERIFIED_FAILURE` | `Verified failure` | Independent evidence establishes that the intended postcondition failed or a defined failure condition occurred. This is not synonymous with the orchestrator state `FAILED`. |

If the policy blocks an action before dispatch, display `Blocked by policy` and `Verification not run`; do not manufacture a verification outcome for an action that never executed. Internally, the outcome field remains absent until an outcome evaluation is applicable. If execution may have caused a side effect but evidence cannot settle the result, record `UNKNOWN`. A task can be `FAILED` with `UNKNOWN` if its runtime crashed before reconciliation, or `COMPLETED` with `VERIFIED_FAILURE` if the workflow terminated normally but its requested goal demonstrably failed. These combinations must be covered by tests. The primary lifecycle label must never conceal a distinct verification outcome.

### 15.6 Crash and resumption protocol

[BA-RECOVERY-001] On crash, browser disconnect, model failure, security hard stop or application restart:

1. Stop dispatching new actions.
2. Invalidate stale targets, epochs, node IDs and pending authorization tokens.
3. Preserve the last safe state/checkpoint without exposing secrets.
4. Record the reason and safe audit metadata.
5. Restore browser/task metadata from local storage if valid.
6. Reconnect and verify browser process/profile/target identity.
7. Re-perceive the actual page and reconcile current state against the saved checkpoint. If page state has drifted, show `The page changed since pause — the plan will be re-checked` before presenting a new plan or action for authorization.
8. Re-run policy checks before the next action.
9. Require an explicit user resume after takeover, sensitive confirmation, or a hard stop.
10. Report uncertain outcomes as `UNKNOWN` until sufficient new evidence exists.

Never replay the last write action simply because its completion record is missing. First establish whether the original side effect already occurred.

---

## 16. Local model and hardware constraints

### 16.1 Supported baseline and runtime decision

The primary reference device is a MacBook Air with an M2 processor and 8 GB unified memory. Windows x64 desktop/laptop is also supported. Manual browsing must remain useful without local inference on both platforms.

Use llama.cpp as the only production inference engine to avoid maintaining separate MLX and llama.cpp inference stacks. Use the Metal backend on Apple silicon when verified with the pinned build. On Windows, use the CPU path as the baseline; enable a GPU backend only for hardware/backend combinations that pass correctness, stability and performance tests. Do not make a particular GPU mandatory for ordinary browsing.

Run the model as a managed local child process using stdin/stdout JSONL or an equivalent OS-scoped private channel. Do not start an HTTP server or unauthenticated localhost endpoint for inference. The Rust supervisor owns process lifecycle, prompt construction, timeout, output validation, cancellation and resource-stop behavior. Model output is untrusted regardless of the process boundary. A watchdog detects a non-responsive runner against the configured inference deadline. A runner exit during an active task transitions the task to `FAILED` with cause `model runner exited`, preserves the last safe checkpoint, and yields `UNKNOWN` if any side effect is unresolved. Restart is offered for the next task only after health/readiness checks; never silently retry the same proposal mid-task.

### 16.2 Model and artifact policy

- Freeze a specific model file, quantization, context limit, tokenizer/config and runtime build only after the benchmark passes.
- Record upstream source, license, cryptographic digest, artifact size, supported hardware, known limits and update/revocation status in a local manifest.
- Model installation is explicit, resumable, cancellable and integrity-checked. Verify the exact downloaded bytes before use. No silent remote inference fallback or automatic replacement with a different model.
- Keep model files in an app-owned data directory; never execute a model-supplied binary or runtime plugin.
- Validate structured output against a closed schema. Unknown fields, oversized output, invalid enums, non-finite numeric values and malformed actions are rejected before policy evaluation.
- Bound prompt/context length, page-observation size, output tokens, inference duration, retries and concurrent generations.

[BA-MODEL-PROFILE-001] Maintain one llama.cpp engine implementation and permit multiple installed, integrity-verified GGUF model artifacts, but only **one resident model per application process**. Choose an approved profile from measured hardware class; do not load/swap models mid-task. [BA-MODEL-ISOLATION-001] Every task gets a fresh inference context/KV cache and explicit bounded task prompt history. Destroy or securely reset the prior task context before accepting the next task; page content and task history must not leak across tasks through retained prompt state. If the pinned runner cannot prove that reset path, restart the runner between tasks and measure the reload cost rather than reusing uncertain context. The 8 GB Apple-silicon profile is `COMPACT`; the Windows 16 GB reference profile is `STANDARD`. Each profile identifies allowed model artifact hashes, runtime build, context limit, quantization, memory cap, output cap and benchmark report. A profile is not bound to a particular model name until it passes the current frozen benchmark. Model selection is explicit and does not silently replace a failing/unavailable model.

[BA-MODEL-REPORT-001] After successful model installation, offer (do not automatically run) a local `Run model readiness check` that measures cold load, warm proposal latency, tokens/second, runner resident memory and one harmless schema-validation task. Display actual measured values and the tested profile/date; never display placeholders or claim a full workflow-success score from this short check. Settings includes `Verify model integrity`, which rehashes the installed artifact with actual progress and reports pass/failure. Neither action uploads results. Keep a replay viewer in diagnostics that presents redacted per-step action/state records and hashes of observation/model-input payloads. Replays are analysis-only and can never dispatch browser actions. Store raw task/page text only where the user explicitly chooses to save a note or export a redacted artifact.

### 16.3 Provisional performance gates

[BA-MODEL-001] These are targets to measure, not claims of achieved performance:

- Model plus inference-runtime resident memory: at most 2.5 GB under the defined benchmark.
- p95 complete structured action-proposal latency: at most 2.0 seconds in the frozen reference workload; report model-load and warm-inference latency separately.
- Vertical-slice workflow success: at least 80% on a frozen task set, with separate reporting for task correctness, policy correctness and independent verification.
- Total Browser-Agent peak memory: at most 6.0 GB on the 8 GB M2 reference device under the defined workload. This is not sufficient by itself: also require no sustained memory-pressure warning, no persistent swap-driven UI stall, and acceptable interactive latency while browsing.
- Run a 20–30 minute thermal soak on the fanless MacBook Air-class reference device and repeat the measured workflow suite afterward.
- Record baseline, model-loaded idle, active inference, multi-tab browsing and peak memory; include relevant child processes and report OS memory pressure/swap separately.
- Measure browser input latency and rendering separately from inference latency. A low model latency does not compensate for a UI or browser that becomes unresponsive.

For Windows, define a reference x64 machine with 16 GB RAM for the local-agent performance gate. Manual browsing must still run at 8 GB where CEF and system constraints permit, but the Agent remains parked if the local model cannot safely fit its resource budget. Do not claim equivalent Windows/Mac performance without independent measurements.

### 16.4 If no model passes

Do not silently relax gates, add a second inference engine, or move inference to a cloud service. Apply this order:

1. Narrow the supported task envelope and favor deterministic, validated skills.
2. Reduce observation/prompt size and context length without removing policy-critical evidence.
3. Reevaluate quantization and llama.cpp runtime configuration, then rerun the frozen benchmark.
4. Park local inference on hardware that cannot meet safe memory/latency budgets; provide honest readiness information.
5. Add more human checkpoints and smaller verifiable steps where useful.
6. Consider a bigger hardware baseline only through a recorded product decision and clear support statement.

The manual browser remains functional when the model is missing, parked, incompatible, overloaded or failed.

## 17. Reusable skills and persistent task data

- Skill/macro entries must be versioned immutably once approved.
- Every skill defines preconditions, target evidence, allowed action types, postconditions and failure/abort behavior.
- Validate current page and origin before replay.
- A reusable skill reduces repeated reasoning; it does not bypass fresh perception, authorization, secure-fill constraints or verification.
- Record success/failure evidence locally and redact secrets.
- Provide a revocation/update pathway; stale skills must fail safely.
- Do not present an unvalidated model-generated macro as a trusted skill.

Persistent app data should include only what is needed: settings, permitted task/checkpoint metadata, local model manifest/integrity state, profiles, skill definitions, audit evidence and locally stored history as designed. Establish retention and clear-data behavior for each category.

[BA-SKILLS-001] Provide a Skills screen where users can inspect a skill's name, immutable version, creator/provenance, source task reference, preconditions, supported origins/target evidence, allowed actions, tested postconditions, last benchmark result and failure/abort behavior. A skill is created only from a task that the user explicitly chooses to save and reviews. Users can disable or delete a skill and revoke it immediately. Display which skill matched when reuse is proposed. Model-generated but unreviewed skills never execute as trusted skills. Any skill update creates a new immutable version and reruns its tests.

[BA-HISTORY-001] Agent task history is local, normal-profile-only and uses the default 90-day retention from Section 15.1. Records include task text, terminal lifecycle, verification outcome, safe event summary and user-selected artifacts; page content is not stored wholesale by default. `Run a similar task again` copies the task text to the composer without starting it, strips any secret-looking literal and requires the user to review. Private-window activity never enters this store. Exporting a task report is explicit, local and redacted; the report lays out requested → proposed → authorized → executed → verified with timestamps, action IDs, policy outcomes and evidence references, but excludes credentials, raw page dumps and unredacted field values.

### 17.1 Website permissions, downloads and external protocols

- Use per-origin permission decisions for notifications, camera, microphone, geolocation, clipboard access and other platform/site capabilities supported by CEF.
- Default to deny or ask before granting any unapproved capability. Permissions are granted by the user through clear app-owned UI; page content cannot approve its own permission prompt.
- Invoke OS-level permission flows only after a user action when required by the platform. Reflect the actual permission state, including denied, restricted and unavailable states.
- Provide a visible way to review and revoke site permissions. Do not grant all permissions globally as an automation convenience.
- External-protocol links (for example, opening another application) require a clear user gate unless explicitly pre-authorized by a narrowly scoped policy. Display the destination and proposed app where available.
- Downloads must use an explicit destination policy. Prevent path traversal and unintended overwrite, report actual completion, and do not automatically execute downloaded files.
- File uploads and open/save dialogs must use native dialogs or scoped file-picker permissions. Never grant remote page content unrestricted filesystem paths.
- Pop-ups, redirects and downloads inherit no Agent authorization merely because the initiating page was approved. Reassess the new target, origin and side-effect class.
- [BA-DOWNLOAD-001] Every Agent-initiated download requires a per-file user approval before writing to disk. Approval presents the initiating origin, canonical filename, content type, known/estimated size, requested destination and overwrite status. Enforce destination/path rules and configured size caps, write to a temporary quarantine name, validate completion and metadata, then atomically move to the selected destination. Never auto-open or execute a downloaded file. Downloads history labels provenance such as `Requested by Agent task <local task ID>`. A site-triggered download is not authorized merely because the page was visited.
- Revalidate the caller/frame origin on every native bridge message, not only when a frame is registered. A frame navigation/context reset invalidates outstanding bridge capabilities. Fuzz the C ABI and native process-message deserializers; test oversized lengths, invalid UTF-8/Unicode, duplicate sequence IDs, stale origins, malformed variants and re-entrant callbacks. The bridge is considered “audited” only when its call-surface inventory, trust-boundary diagram, ownership/thread-affinity review, schema validation, negative authorization tests, fuzz/sanitizer results and CODEOWNERS review artifact are present. [BA-BRIDGE-002]

---

## 18. Accessibility and input behavior

### 18.1 Interaction

[BA-A11Y-001] Every interactive control must support keyboard use and expose an accessible name/role/state.
- [BA-A11Y-002] Approval and high-risk handoff dialogs place keyboard focus on `Cancel` (or the safest non-approving option) by default; focus never defaults to Approve. On dialog close, return focus to the control that opened it unless that target no longer exists, then return it to the owning Agent panel.
- Map lifecycle changes to the accessible status/live-region copy from BA-STATE-001; coalesce repetitive noncritical announcements within 500 ms to avoid screen-reader spam, but announce newly required user action, policy blocks and safety stops immediately. A state is also visible as text, never only in a live region or color.
- A target outline must have a parallel text description such as `Target: Submit application form`; visuals never serve as the only target/status indication.
- Run both dark and light OS appearance themes, reduced motion enabled/disabled, keyboard-only use, VoiceOver on macOS and a supported Windows screen reader. Website content accessibility remains website-controlled, but the app shell, native bridge transitions and cross-surface focus path must be tested.

- Keep the focused control visibly identifiable using a solid, surface-appropriate outline.
- Do not rely on hover-only discovery for required actions.
- Keep hit targets comfortable for mouse, trackpad and high-DPI display use. Follow platform guidance while respecting the compact browser toolbar.
- Preserve familiar browser shortcuts and do not steal shortcuts from text entry or OS behavior unnecessarily.
- Use platform-appropriate modifiers: `Command` on macOS, `Control` on Windows where appropriate. Write shortcut names as text; do not print macOS command glyphs.

### 18.2 Core shortcut baseline

Where feasible and consistent with native desktop behavior:

| Action | macOS text form | Windows text form |
|---|---|---|
| Focus address bar | `Command + L` | `Control + L` |
| New tab | `Command + T` | `Control + T` |
| Close tab | `Command + W` | `Control + W` |
| Reopen closed tab | `Command + Shift + T` | `Control + Shift + T` |
| Reload | `Command + R` | `Control + R` |
| Next tab | `Control + Tab` or native equivalent | `Control + Tab` |
| Previous tab | `Control + Shift + Tab` or native equivalent | `Control + Shift + Tab` |
| Cancel/stop current relevant operation | `Escape` | `Escape` |

Implement according to tested platform conventions. Document the actual behavior in Settings. Do not show unsupported shortcut hints.

### 18.3 Color and state accessibility

- Meet WCAG 2.2 contrast requirements for all first-party text and applicable interface components.
- Never use color alone for success, failure, waiting, blocked, selected or unknown states.
- Ensure status text is readable in all states, including on Deep Teal and raised surfaces.
- Check focus indication against its adjacent backgrounds and never use glow as a focus substitute.

### 18.4 System preferences

Respect reduced motion and platform text scaling where supported. Preserve keyboard focus when panels reflow. Provide a visible focus order across browser chrome, webpage surface and Agent panel. Focus movement into the webpage and back to the app shell must be predictable.

Test the first-party UI with VoiceOver on macOS and a current Windows screen reader such as Narrator. Remote website accessibility remains subject to the website itself, but Browser-Agent must not intercept, obscure or disable platform assistive technology. Test at least WCAG 2.2 AA criteria applicable to the product UI, including text contrast, non-text contrast, keyboard operation, focus visibility and focus not obscured, target size minimum (including documented exceptions), accessible names/roles/states, reflow/text scaling and reduced motion. WCAG conformance is not claimed solely from automated checks.

---

## 19. Error and status copy

All visible errors are written for users first. Diagnostic codes may be available in a secondary details section.

Examples:

- **Chromium failed to start:** Explain that Browser-Agent could not start its managed browser and give a real retry/log action.
- **Website failed to load:** Keep the real browser error context; do not display a fake page.
- **Agent cannot act:** Say whether the model is unavailable, the target is ambiguous, policy blocked the action or the page changed.
- **Confirmation required:** Explain the action and why user approval is necessary.
- **Verification inconclusive:** Say the action may have happened but cannot be confirmed.
- **Resource stop:** Say the Agent stopped to protect a safe state, what is known and what needs user attention.
- **Audit integrity failure:** Do not claim the audit chain is verified.

Avoid raw stack traces in the normal view. Provide a controlled diagnostic route for detailed error information, with secrets redacted.

---

[BA-ERROR-COPY-001] Each visible error follows the four-part contract: attempted action; observed event; what is known/unknown now; safe next action. Required worked examples cover navigation, model not ready/runner crash, policy block, approval expiry, unknown verification, download rejection, audit-write failure and resource stop. Avoid generic “Something went wrong” or “Done” where the evidence does not justify it. Error IDs, stack traces and raw CEF/model diagnostics stay in the redacted diagnostic view.

## 20. Build, integration and repository discipline

### 20.1 Real implementation over component theatre

- Do not replace functionality with a screenshot, static prototype or fake data while claiming the feature is complete.
- Do not build a separate frontend-only browser simulation that bypasses Rust/runtime authority.
- Do not reintroduce Tauri/Wry or a separate stock Chromium process into the production rendering path without a reviewed architecture decision; the selected single-CEF composition exists to keep the real page and app shell in one supported native view hierarchy.
- Do not create controls that update only local component state when the underlying operation should reach Rust.
- Do not add packages just because they appear in a template. Keep dependency weight, supply-chain risk, memory use and startup latency under review.
- Reuse verified repository components; remove dead code only after confirming that it is genuinely unreachable/obsolete.
- Preserve backend security code and architecture during visual cleanup unless an explicit technical decision authorizes a change.
- Do not mark unfinished required work as “dead code.”

### 20.2 Runtime/frontend contract

Define typed commands/events for native window/view geometry, browser state, tabs, navigation, Agent task lifecycle, intervention, policy results, downloads, verification, audit and model readiness. Validate every C ABI, CEF message-router, IPC and TypeScript boundary payload at runtime, not only at compile time. Every event carries a schema version, monotonic event sequence, relevant browser/tab/frame identity and task/action ID where applicable. Unknown fields, stale sequence values, invalid states or duplicate commands must not grant authority. The first-party local UI bridge is unavailable to remote page contexts.

### 20.3 Cross-platform and security build rules

- Test macOS and Windows separately.
- Preserve Chromium sandboxing and site isolation.
- No `--no-sandbox`, `--disable-web-security` or `--disable-site-isolation-trials` flags.
- Do not expose CDP, DevTools discovery or any other browser-control endpoint in production. CDP is disabled in release builds and reserved for disposable dev/test fixtures.
- Avoid unnecessary shell capabilities, broad filesystem permissions and unsafe IPC exposure.
- Audit dependencies and pin/verify production artifacts according to repository policy.
- Pin CEF and Chromium versions to a tested release branch. Produce a dependency inventory and the applicable CEF/Chromium third-party license notices. Verify CEF subprocess/resource packaging on every supported platform.
- Package real release outputs: a signed/notarized macOS `.dmg` and a signed Windows installer such as NSIS `.exe` or MSI, as selected by the release process.
- Do not claim signing, notarization, reproducibility or integrity verification until the relevant pipeline proves it.

### 20.4 Documentation traceability

[BA-TRACE-002] Every release-critical requirement must use the stable inline ID system in Section 0.6 and be present in Appendix D's requirement registry and the repository's evidence matrix. For each ID, record the owning module, automated/manual test ID, proof artifact path or build URL, test result/date, status (`UNVERIFIED`, `IMPLEMENTED_UNTESTED`, `TESTED_FAILED`, `PASS`, or `ACCEPTED_LIMITATION`) and deviation owner/expiry where applicable. A `PASS` status requires evidence from an executed test on the stated target; code inspection and a successful build alone do not qualify for behavioral/security requirements.

Keep this specification, token file, generated contracts, workflow definitions and tests consistent. The CI contract-drift job must fail if required IDs lack a matrix entry or the matrix points to unknown IDs. When implementation intentionally differs from a requirement, record a reviewed ADR and rationale instead of silently diverging. Do not invent a second ID scheme in an implementation task.

### 20.5 Required repository tooling and single-source configuration

The repository must make local checks and CI checks identical. Check in these files; do not rely on developer-global formatter, linter or compiler settings:

```text
.github/
  workflows/
    ci.yml
    security.yml
    release.yml
  dependabot.yml
  CODEOWNERS
  pull_request_template.md
.editorconfig
.gitattributes
.gitignore
.prettierrc.json
.prettierignore
eslint.config.mjs
rust-toolchain.toml
rustfmt.toml
.clang-format
deny.toml
ui/package.json
ui/tsconfig.app.json
SECURITY.md
CONTRIBUTING.md
THIRD_PARTY_NOTICES.md
Cargo.toml
Cargo.lock
package.json
package-lock.json
.npmrc
.node-version
```

Keep the repository's existing layout if it already has equivalent files; do not create duplicate competing configurations. The root `Cargo.lock` and frontend `package-lock.json` are committed. Production and CI installs must use locked/frozen dependency resolution. Dependency updates are explicit reviewed changes, not an automatic side effect of a build.

- **Initial Rust baseline:** pin Rust **1.99.0** in `rust-toolchain.toml` (the stable release announced on 2026-10-01), with `rustfmt` and `clippy` components and `profile = "minimal"`. Use Rust Edition 2024 and set the initial shipped crates' `rust-version = "1.99"`. Lower the MSRV only through a tested compatibility PR; do not claim compatibility with an older compiler without running the MSRV job. The scheduled latest-stable lane detects future breakage but does not silently update the pinned release toolchain. [Official release](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/)
- **Initial Node baseline:** pin **Node.js 24.21.0 LTS** in `.node-version`, and constrain `package.json` with `"engines": { "node": ">=24.21.0 <25" }`. This is the latest LTS patch verified during the 2026-10-13 research pass; Node 26 is Current, not the selected production line. Upgrade to a newer Node 24 security patch through a reviewed PR, and move to a new LTS major only after validation. [Official Node 24.21.0 release](https://nodejs.org/en/blog/release/v24.21.0)
- Use npm and the checked-in `package-lock.json`; run `npm ci` in CI. Do not mix npm, pnpm, Yarn and Bun lockfiles in one frontend workspace.
- Pin the CEF distribution and `llama.cpp` source/build to exact reviewed versions/commits and hashes. Do not compile release binaries from floating branches or download unverified artifacts.
- Use `.gitattributes` to normalize text files to LF. Keep model files, browser profiles, credentials, databases, audit data and generated platform binaries out of source control.
- Keep all formatting and linting commands documented in `CONTRIBUTING.md` and exposed as package scripts or repository scripts. The CI commands are the source of truth.

### 20.6 Rust coding rules

Rust is the trusted policy and orchestration layer. Rules in this section are mandatory for production code.

#### Safety and FFI

- No `unsafe` in application, policy, perception, executor, verifier, model-runtime supervisor, audit, vault or storage crates. Those crates use `#![forbid(unsafe_code)]`.
- Isolate required FFI and unsafe operations in the smallest possible `cef-bridge` crate/module. The bridge must not own policy decisions, credential values or action authorization.
- In the FFI crate, enable `#![deny(unsafe_op_in_unsafe_fn)]`. Every unsafe block must have an adjacent `// SAFETY:` comment that states the specific pointer, lifetime, aliasing, thread-affinity and ownership invariant being relied on. Do not use blanket lint suppressions.
- C++ exceptions and Rust panics must never unwind across the C ABI or CEF callback boundary. Catch/translate failure within the owning language boundary and return a bounded error result; if recovery is impossible, terminate the affected helper safely rather than continuing with corrupted state.
- All CEF API calls must obey CEF thread-affinity requirements. Rust must not block the UI/CEF thread on model inference, disk operations, network waits or locks. Debug builds assert CEF thread affinity at every native call boundary; release builds return a typed failure or stop the affected operation safely on an invalid-thread call. Instrument bridge enqueue-to-completion latency locally and include it in performance profiles. Do not invoke callbacks into React synchronously from a CEF-sensitive thread.
- Treat C ABI and IPC payloads as hostile input. Validate lengths, enum values, sequence numbers, IDs and schema versions before use; cap payload sizes before deserializing.

#### Error handling and concurrency

- Use typed error enums (`thiserror` or equivalent) in library/domain crates. Add context at subsystem boundaries without logging secret material. `anyhow` may be used at application composition boundaries, not as a substitute for typed policy/IPC outcomes.
- Do not use `.unwrap()`, `.expect()`, `panic!`, `unreachable!` or unchecked indexing in production paths reachable from a page, model output, IPC, file, network event or user action. Tests may use them when failure messages remain useful.
- Do not convert an error into success-shaped defaults. An absent or malformed policy decision, verification result or IPC response must fail closed.
- Do not hold a mutex/RwLock across `.await`. Use bounded channels, explicit ownership and documented lock ordering. Define queue/back-pressure limits for browser events and model requests.
- Every wait, retry and inference call must have a deadline and cancellation path. Retries must be idempotent or preceded by a fresh side-effect reconciliation; never blindly retry writes.
- Use structured tracing. Redact credentials, cookies, auth headers, OTPs, form contents, model prompts that may contain private data, and credential-bearing URLs before event construction, not merely at log formatting time.
- Use typed wrappers for secrets and opaque references. Avoid `Debug`, `Display`, serialization and `Clone` implementations for secret-bearing types unless each behavior is intentionally safe. Use zeroization where practical, while documenting that copies made by the OS, allocator or target page cannot be promised to disappear instantly.

#### Types, boundaries and tests

- Prefer enums and newtypes over stringly typed states, action classes, origin identifiers, authorization tokens and verification outcomes.
- IPC/action enums use explicit tagged serialization. Reject unknown command variants, unknown fields where the schema is closed, duplicate or stale sequence IDs, oversized payloads and schema-version mismatches. Unknown data may never grant authority.
- Avoid global mutable state. Keep browser profile identity, task lifecycle, authorization, secret storage and verification result ownership explicit.
- Do not expose arbitrary JavaScript execution, shell execution, generic filesystem access or arbitrary native command names to the model or webpage. The executor accepts only the typed allow-listed action schema.
- Add unit tests to each crate; integration tests to crate boundaries; property-based tests for state transitions, parser/serializer round trips, authorization binding and redaction; and fuzz targets for untrusted FFI/IPC payloads and schema parsers where practical.
- Test negative cases: missing authority, wrong origin, stale epoch, replayed authorization, malformed schema, timeout, cancellation, partial write, event reordering, panic/error at a bridge boundary and audit-write failure.
- Avoid blanket `allow` attributes. Every exception needs a narrow scope, rationale, issue/requirement ID and removal condition.

#### Rust lint baseline

- All non-FFI production crates must compile warning-free.
- CI runs `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` using the pinned toolchain.
- Enable Rust's `unsafe_code` restriction workspace-wide and explicitly configure only the FFI crate exception. Enable `unsafe_op_in_unsafe_fn` for the FFI crate.
- Do not enable every `clippy::pedantic` lint blindly. Start with the stable correctness/performance lints, run the pinned Clippy configuration, and add individual additional lints with a short documented rationale. Warnings must not be silenced project-wide to make CI green.
- Use `cargo test --workspace --locked` plus explicit feature/target combinations that the product actually ships. Do not assume `--all-features` is valid if mutually exclusive features exist.

### 20.7 TypeScript and React coding rules

The UI is an untrusted presentation client even though its source code is first-party. It never owns authorization or trusted runtime state.

- Enable TypeScript `strict` and at minimum `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `noImplicitOverride`, `noFallthroughCasesInSwitch`, `forceConsistentCasingInFileNames`, `noEmit` and consistent module syntax. Any exception must be documented and narrow.
- Do not use `any` in app code. Prefer `unknown` followed by runtime validation. Do not use `@ts-ignore`, broad ESLint disables or non-null assertions to bypass uncertain data; a suppression must cite its reason and tracked follow-up.
- Use ESLint flat config with `typescript-eslint` typed lint rules, React Hooks rules and React Refresh checks. Type-checking and linting are separate required CI jobs/commands; a passing build is not a substitute for either.
- Use React Testing Library and Vitest for component/unit tests. Keep Playwright or other browser automation as test-only tooling; it must not be shipped as the production browser-control runtime.
- Never store credential values, authorization tokens, private browser content or vault material in React state, browser localStorage/sessionStorage, analytics or client logs. React receives only safe display data and opaque identifiers.
- Do not place page-controlled content into trusted app UI as raw HTML. `dangerouslySetInnerHTML`, `innerHTML`, `eval`, `new Function` and dynamic execution of page/model strings are prohibited in the first-party UI. If a future, reviewed feature genuinely requires rich HTML, it needs allow-list sanitization, a Trusted Types policy where supported and dedicated XSS tests; escaping alone must not be assumed to sanitize arbitrary HTML.
- Do not load first-party app code, fonts, scripts or styles from a CDN at runtime. Bundle and pin them locally. Apply a restrictive Content Security Policy to `browser-agent://ui/` and keep remote browsing contexts separate from the privileged app scheme.
- Every bridge result from native code is runtime-validated before use. TypeScript interfaces alone are not runtime validation. Validate discriminated unions, origin/task/frame identities, schema versions and payload bounds.
- Expose a small explicit bridge API (`browser.navigate`, `agent.submitTask`, `agent.pause`, and similar documented methods). Do not create a generic `invoke(command: string, payload: any)` API or expose the entire native command surface to React.
- Keep status derived from native/runtime events. No synthetic timers, guessed task outcomes or frontend-owned copies of policy/task state.
- Make effects cancellable and clean them up on unmount. Guard against stale async results overwriting a newer task, tab or observation epoch.
- Keep tests for user-visible status, keyboard/focus behavior, bridge validation, rejected commands and safe error rendering.

### 20.8 Formatting, linting and editor settings

Use one formatter per language; do not ask Prettier to format Rust or C++.

- Rust: `rustfmt` governed by root `rustfmt.toml` and the pinned toolchain.
- TypeScript, TSX, JavaScript, CSS, JSON, JSONC, Markdown and YAML: Prettier governed by `.prettierrc.json`.
- TypeScript correctness: TypeScript compiler plus ESLint/`typescript-eslint`; Prettier is not a linter and must not be used as the correctness gate.
- Native C/C++: `clang-format` with root `.clang-format`, plus `clang-tidy` and compiler warnings. Formatting is not a substitute for static analysis.
- Keep ESLint and Prettier responsibilities separate. Use `eslint-config-prettier` to disable conflicting stylistic lint rules. Do not run an extra Prettier ESLint plugin that duplicates formatting work.
- A developer pre-commit hook may format changed files and run fast checks, but hooks are convenience only; CI must rerun every required check because hooks can be bypassed.

Use these canonical formatting decisions:

```json
{
  "semi": true,
  "singleQuote": true,
  "trailingComma": "all",
  "printWidth": 100,
  "tabWidth": 2,
  "useTabs": false,
  "endOfLine": "lf",
  "proseWrap": "always"
}
```

The config lives in `.prettierrc.json`. Check in this exact initial `rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.99.0"
components = ["clippy", "rustfmt"]
profile = "minimal"
```

Use this conservative root `rustfmt.toml` and keep it compatible with the pinned stable formatter:

```toml
edition = "2024"
style_edition = "2024"
max_width = 100
use_small_heuristics = "Default"
```

Rust and C++ retain their own language-native conventions. `.editorconfig` sets UTF-8, LF, a final newline and trailing-whitespace trimming; it uses two spaces for TypeScript/JSON/CSS/YAML/Markdown, with Rust formatting governed by rustfmt and native C++ by clang-format. Do not create competing per-package formatting configurations unless the package has a documented technical need.

Use a root npm workspace so one `package-lock.json` governs the frontend toolchain and UI dependencies. The root `package.json` owns Prettier and ESLint; `ui/package.json` owns the Vite app, React, TypeScript and Vitest. Required root scripts are:

```json
{
  "workspaces": ["ui"],
  "scripts": {
    "format": "prettier . --write",
    "format:check": "prettier . --check",
    "lint": "eslint ui",
    "typecheck": "npm --workspace @browser-agent/ui run typecheck",
    "test": "npm --workspace @browser-agent/ui run test",
    "test:ci": "npm --workspace @browser-agent/ui run test:ci",
    "build": "npm --workspace @browser-agent/ui run build"
  }
}
```

The `ui/package.json` must use the exact workspace name `@browser-agent/ui` and define `typecheck` as `tsc --noEmit -p tsconfig.app.json`, `test` as `vitest`, `test:ci` as `vitest run`, and `build` as `vite build`. Adapt paths to the repository's actual Vite layout but preserve the separate gates. Add stylelint only if the CSS/SCSS surface grows enough to justify it; palette/token enforcement is required regardless. Exclude generated files, `node_modules`, build outputs, third-party CEF sources and binaries, model files and packaged assets from formatting scans.

### 20.9 Continuous integration — required pull-request gate

[BA-CI-001] Use GitHub Actions as the canonical CI provider. Every pull request and every push to the protected main branch runs the same required checks. Workflows are checked in and reviewed like application code.

#### Rust quality job

Run from the workspace root using the pinned toolchain:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
cargo doc --workspace --no-deps
```

Also run explicit tests for the supported feature/target combinations and the minimum Rust version where claimed. Unit tests should run on a fast Linux runner when platform-independent; CI must also compile and test the Rust crates on macOS and Windows. Platform-sensitive behavior is not considered covered solely because Linux tests pass.

#### TypeScript/React job

Use the pinned Node version and root workspace lockfile from the repository root:

```sh
npm ci
npm run format:check
npm run lint
npm run typecheck
npm run test:ci
npm run build
npm audit --workspaces --audit-level=high
```

Fail CI on formatting drift, ESLint errors, type errors, test failures, production build failures or high/critical dependency vulnerabilities. `npm audit` does not replace review of transitive native components or license compliance.

#### CEF/C++ native job

- Compile the actual bridge/host against the pinned CEF distribution on macOS Apple silicon and Windows x64.
- Run `clang-format` check-only mode and `clang-tidy` using a generated `compile_commands.json`.
- Enable strong compiler warnings and treat project-code warnings as errors; isolate unavoidable third-party warnings rather than disabling warnings globally.
- Run native bridge unit tests and C ABI serialization tests on both OS targets. Run AddressSanitizer/UndefinedBehaviorSanitizer or the closest supported platform instrumentation in dedicated jobs.
- Exercise CEF startup, one-window layout, native input, view resizing, scheme isolation, shutdown/restart and crash handling in real platform smoke tests. Use a deterministic local test page, not public websites, for repeatability.

#### Contract and integration job

- Regenerate JSON Schemas and TypeScript types from the authoritative Rust contract definitions; fail if generated outputs differ from checked-in outputs.
- Test serialization/deserialization round trips and rejection of unknown commands, fields, versions, duplicate sequence IDs and oversized payloads.
- Run end-to-end tests against the real CEF host in dev/test mode, with a disposable profile and no production CDP listener.
- Upload test reports, coverage and crash diagnostics as artifacts only after redaction checks; artifacts must not contain browser profiles, credentials, model prompts or unredacted page snapshots.

#### Required workflow set

- `.github/workflows/ci.yml`: formatting, linting, type checks, unit/integration tests, supported-platform build checks and app bundle smoke tests.
- `.github/workflows/security.yml`: static analysis, dependency/security scans, secret scan, license/source policy and scheduled scans.
- `.github/workflows/release.yml`: protected release builds, signing, notarization, SBOM and provenance attestation; never run release/signing logic on untrusted pull requests.

If CEF build size or duration is expensive, caching may accelerate the build, but only verified upstream CEF archives pinned by version and checksum may be restored. Do not execute caches or artifacts supplied by untrusted pull-request workflows in a privileged release workflow.

### 20.10 Security scanning and supply-chain policy

[BA-SUPPLY-001] Security scans run on pull requests, protected-branch pushes, and on a scheduled basis at least weekly. New critical findings are handled immediately; do not wait for the next weekly job.

Required controls:

- **Rust dependencies:** `cargo audit` against the RustSec Advisory Database and `cargo deny check` for advisories, unreviewed licenses, duplicate/banned crates and disallowed dependency sources. Keep `Cargo.lock` committed. No wildcard git dependencies or unpinned git revisions in production.
- **JavaScript dependencies:** `npm audit --audit-level=high` against the lockfile, plus automated dependency update PRs. Review native/transitive dependency and postinstall scripts rather than trusting the audit score alone.
- **Cross-ecosystem:** OSV-Scanner or an equivalent lockfile/SBOM scanner to cover Cargo, npm and packaged native components. Reconcile duplicate scanner findings rather than treating each tool's count as separate vulnerabilities.
- **Static analysis:** CodeQL for Rust, JavaScript/TypeScript, C/C++ and GitHub Actions workflows when the repository/plan supports the required analysis. C++ analysis must build the native target; a missing compilation database is a failed analysis, not a clean result.
- **Secrets:** Gitleaks in pull-request and history-aware checks, with platform secret scanning/push protection enabled where available. If a real secret is found, revoke/rotate it first; deleting the line is not sufficient.
- **Workflow hardening:** pin every third-party GitHub Action to a reviewed full-length commit SHA, not a mutable branch or version tag. Set workflow/job `permissions` to the minimum needed, defaulting to read-only. Use explicit allow-lists for actions when available.
- **Untrusted PRs:** never expose signing keys, vault credentials, deployment tokens or repository write tokens to workflow runs that execute contributor-controlled code. Do not use `pull_request_target` to check out and run untrusted pull-request code.
- **Release authority:** put signing secrets in a protected release environment with approval rules. Give only the release job the required write/attestation permissions. Prefer short-lived OIDC-based credentials when a supported service allows them; never put long-lived signing keys in the repository.
- **Dependency review:** inspect every manifest/lockfile change for newly introduced packages, advisories, licenses, install scripts, project health and whether the dependency is actually necessary.
- **Inventory and provenance:** generate a software bill of materials (SBOM) for each release and attach a signed build-provenance attestation where supported. Include CEF/Chromium, `llama.cpp`, fonts, native libraries, model manifest and license notices in the component inventory. Do not claim that an SBOM alone establishes that an artifact is safe.

Severity policy:

- A reproducible critical security issue or a critical authorization/privacy bypass blocks merge/release until fixed or until the vulnerable code path is removed from the artifact.
- A high-severity reachable vulnerability blocks release by default. A temporary exception must name the owner, affected component/path, exploitability rationale, mitigation, tracking issue and expiry date; it must not waive a critical policy invariant.
- Medium/low findings are tracked and triaged with a documented fix/accept decision. Unknown license, unverified binary origin or hash mismatch blocks packaging.
- Never use blanket ignores such as `allow-all`, never suppress a scanner finding without a scoped documented exception, and never report a clean bill of health solely because a tool returned no finding.

License allow/deny rules must be checked against the real Cargo/npm/CEF/Chromium/llama.cpp/asset inventories before release. Deny unknown or missing license metadata by default, generate all required notices, and do not automatically accept GPL/AGPL/SSPL or any other copyleft/commercially constrained license without a documented compatibility review. Do not assume CEF's own license file covers every bundled third-party component.

### 20.11 Continuous delivery, releases and updates

[BA-RELEASE-001] CI artifacts from a pull request are validation artifacts, not release candidates. Release candidates are built from a reviewed protected tag/commit through the protected release workflow.
- Build the macOS app bundle and `.dmg` on a supported macOS runner; sign and notarize them using the project's protected signing identity. Build the Windows installer on a Windows runner and sign it using the approved code-signing mechanism. Signing and notarization are release gates, not manual claims.
- Verify the checksum and identity of downloaded CEF/Chromium components, the model runner, third-party libraries and model artifacts. Record the exact source release/commit and hash in the release manifest.
- Produce SHA-256 checksums, SBOM, third-party notices, build provenance and test/security summaries for every release. Keep a recoverable record connecting each distributed artifact to the commit, workflow run, dependency lockfiles and platform.
- Model weights are distributed separately from the base installer. Model download is explicit, hash-verified and activated atomically only after successful validation. Release metadata must identify the model license and exact artifact hash.
- Browser engine security updates are expedited through the same automated compatibility, security and packaging gate. If the current CEF release cannot be updated without a blocking regression, block release or disable the affected capability; do not silently keep shipping a known critical vulnerability.
- Update metadata and download endpoints are integrity-checked. No remote update, model or inference fallback can run without the documented user-visible behavior.
- Maintain rollback instructions that never roll back to a browser engine with a known exploited critical vulnerability. Preserve user data during rollback and define compatible migrations.
- Test uninstall behavior separately from “clear browsing data.” Uninstall must follow the documented product choice for retaining/removing profile, model and audit data; never silently delete user downloads.

### 20.12 Branch protection and dependency maintenance

- Protect the main branch; prohibit force pushes and direct unreviewed updates.
- Require green named checks for Rust, frontend, native CEF, security, contract drift and supported-platform build jobs before merge. Give every required check a unique stable name.
- Require conversation resolution and a reviewed pull request. Require CODEOWNERS review for changes to policy, vault/secure-fill, CEF bridge, CI workflows, dependency allowlists, signing, release and security configuration. If the project is maintained by one person, keep CODEOWNERS anyway to make sensitive ownership explicit for future contributors.
- Use Dependabot or an equivalent reviewed update bot for Cargo, npm and GitHub Actions. Batch routine updates where safe; isolate CEF/Chromium and `llama.cpp` updates because they can alter security boundaries and native behavior.
- Run a scheduled compatibility lane against the latest Rust stable and Node LTS to catch upcoming breaks, while release builds remain pinned to the repository's explicitly supported versions. Changes from this lane do not silently change production versions; they create a reviewed update task/PR.
- On a failed required check, the code is not mergeable. Do not convert required jobs to `continue-on-error` to make the pipeline green. An exception needs a recorded limitation and an owner, and may not bypass a critical security or authorization invariant.


---

[BA-FAULT-001] The native/runtime test harness must inject: disk-full or permission-denied during required audit write; network loss during a page action; renderer-process termination during observation; CEF browser-process restart; model-runner exit during proposal generation; corrupted/truncated model file; private-context close while resources are active; and task-budget exhaustion with a write in flight. Each fault has an expected task state, verification outcome, user copy, audit record expectation and recovery test ID. Failure to write required audit evidence is fail-closed.

[BA-REQ-LINT-002] CI validates the generated requirement registry against the authoritative `requirements/registry.yaml`, checks each release-critical ID has a test ID, flags duplicate/missing/orphan IDs, and fails when Appendix D is stale. The CI must also lint first-party UI string resources for copy-contract violations and assert that the bridge's eager method list in Section 9.1 matches the typed TypeScript/Rust contract in Section 20.7.

[BA-UPDATE-001] Application/CEF updates are verified against a signed manifest, pinned provenance and cryptographic hash before installation. Automatic update checks are disabled by default; Settings exposes a manual `Check for updates` action. Downloads are staged, verified and installed atomically with rollback instructions. Do not roll back to a CEF release with a known actively exploited critical vulnerability. Model updates are separate, explicit and hash-verified; no model or runtime is silently replaced. Crash reports stay local; a user may create a previewed/redacted export but Browser-Agent never uploads one.

## 21. Validation plan and definition of done

The product is not complete because components exist or the build succeeds. Completion requires successful builds, executed tests, visible rendering inspection and functional validation.

### 21.1 Required visual states

Capture and inspect screenshots at realistic sizes for:

1. Default first launch showing the local new-tab page and persistent Agent panel without an unsolicited external request, in both OS-following themes.
2. Ordinary browsing with Agent idle and model unavailable.
3. Agent actively performing a genuine supported workflow.
4. Agent waiting for a user confirmation.
5. Human takeover and explicit resume state.
6. Verified success with supporting evidence.
7. Unknown/inconclusive verification.
8. Actual browser/navigation error.
9. Settings/secondary screen with the same token system.
10. Narrow window with stacked browser/Agent layout.
11. Long tab titles, long URLs, large text and a full task activity list.
12. Loading/crashed/audio-playing tab text states; textual download progress.
13. Approval pending, expired approval, budget exhaustion, stuck detection and blocked-state drill-in.
14. Find in Page, local notes, skill management, task history/report export and model integrity/readiness screen.

No required state may be represented only by a mockup or fabricated sample.

### 21.2 Required functional tests

- First launch and CEF process lifecycle, including the local new-tab page and the absence of unsolicited external requests.
- Model-readiness state-to-copy tests, approval TTL/single-pending enforcement, same-window second-task rejection and task-budget transition.
- Task-owned tab boundaries, popup handling, task-tab cleanup and tab tear-off blocking during active tasks.
- Agent egress classification for user text, task text, URL parameters, vault-derived patterns and changing form destinations.
- Clipboard single-use permission, masked-value audit event and hostname-typed high-risk grant flow.
- Agent-initiated download approval, path traversal/overwrite rejection, quarantine and no-auto-open behavior.
- Sleep/wake re-stabilization, no-focus-steal behavior and pause/cancel/takeover action-boundary semantics.
- Dark/light appearance, accessible status announcement mapping, focus-safe approvals and copy-contract lint.
- Startup, policy, observation, model warmup and memory-governance SLOs with real profiling artifacts.
- Audit/disk-full fault injection, model-runner crash, renderer kill and network loss recovery.
- Actual rendered webpage is the active CEF browser view receiving the same input as the visible page.
- URL entry, search, back, forward, reload, tabs, multiple windows, New Private Window, close/reopen where supported, bookmarks and downloads.
- Permanent Agent panel is visible and actionable at each supported layout size.
- Agent tasks are submitted to the real runtime and statuses reflect real events.
- Model-unavailable path leaves manual browsing functional.
- Task cancellation, pause, intervention, takeover and safe resumption.
- Typed action schema validation and rejection of malformed model output.
- Policy authorization bound to action ID, target/origin/frame/epoch/parameters, expiry and one-time consumption.
- Stale target/node rejection, replay rejection, redirect invalidation and re-perception after changes.
- Proof that the privileged `browser-agent://ui/` bridge is inaccessible to all remote origins and frames, including nested cross-origin iframes.
- Proof that app UI storage is isolated from website cookies and that no user's default Chrome profile is opened or modified.
- Cross-origin frame/OOPIF, shadow DOM, renderer-context replacement and accessibility-tree cases in the supported corpus.
- Secure-fill redaction and exact-origin checks; manual password-manager save/fill behavior; private-window credential disablement; and proof secrets do not appear in frontend state, logs, prompts, screenshots or ordinary audit records.
- Confirmation gate and no-autonomous-payment behavior.
- Independent verification for success, failure and unknown outcomes.
- Resource hard stop and safe checkpoint recovery.
- Audit HMAC continuity, rotation and integrity failure handling.
- Zero background telemetry/analytics/crash uploads; explicit diagnostic-export preview and redaction.
- Cross-context private-browsing storage isolation and disk-artifact inspection after closing the private context, including proof that private suggestions/forms/tasks are not derived from normal-profile state.
- State-to-copy mappings for all lifecycle states/wait reasons and the independent verification-outcome vocabulary.
- Undo/Redo semantics, compensation verification, stale state rejection and redo reauthorization.
- Multi-window task exclusivity and recovery when the owning window closes.
- Localization-readiness tests for RTL, CJK, Devanagari, pseudo-localization, Unicode/IME and font coverage.
- The exact 900 × 640 client-area geometry contract and site usability test cases at a 360 px browser viewport.
- Frozen livelock calibration suite and held-out false-stop/detection results.
- Absence of third-party extension installation or arbitrary extension execution surfaces.
- First-party UI respects OS light/dark appearance and ships no app-owned UI sounds.
- Toolbar width-budget tests at 960–1150 px and responsive stacked-mode Agent-height formula tests.
- Lazy loading does not delay first useful interaction or security-critical controls.
- Keyboard navigation, focus visibility, contrast and reduced-motion behavior.
- Install/build, code-signing, clean-profile, process-lifecycle and packaged-window composition tests on macOS Apple silicon and Windows x64.

### 21.3 Accessibility and design gates

- No app-owned icon outside the whitelist.
- No sans-serif fonts in first-party Browser-Agent UI.
- No manually rendered macOS command-key glyphs.
- No glow, glass, decorative gradient, ambient shadow or unneeded animation.
- Corner radii are selective and within the documented tokens.
- Every text/background pair passes the contrast matrix.
- Every required interactive boundary and focus indicator passes the applicable non-text contrast check.
- Applicable WCAG 2.2 AA tests include target size/spacing, keyboard operation, focus visibility/not obscured and text scaling; VoiceOver and Windows screen-reader smoke tests are documented.
- No clipping, overlap, accidental horizontal scroll in the app shell or unreadable status text.
- Persistent Agent panel cannot be hidden or dismissed.
- No fake states, data, buttons, progress or success labels.

### 21.4 Performance gates

Record actual native-window startup readiness, CEF browser-view readiness, baseline and peak memory across all child processes, system memory pressure/swap, local inference latency, UI input responsiveness, browser rendering/input synchronization and task-level success. Measure model gates from Section 16 on the reference Mac and Windows hardware and include a sustained thermal run on the MacBook Air. Include cold start, warm browsing, multi-tab and model-unavailable scenarios. Performance results must include workload definitions, pinned build/model identifiers, raw logs and summary statistics.

### 21.5 Release gate

[BA-DONE-001] A release candidate must include:

- successful clean builds for supported platforms;
- executed test results;
- visual screenshots for required states;
- current token contrast results;
- security evaluation artifacts for the applicable corpus;
- dependency/package audit artifacts required by repository policy;
- passing Rust formatting, lint, test and documentation jobs;
- passing Prettier, ESLint, TypeScript typecheck, frontend tests and Vite production build jobs;
- passing native CEF/C++ format, static-analysis, supported-platform build and smoke-test jobs;
- passing security, secret-scan, dependency/license and schema-drift jobs with no expired suppressions;
- signed macOS/Windows package evidence, SBOM, third-party notices and build provenance for a distributed release;
- documented known limitations and safe fallback behavior;
- no unapproved deviations from this specification.

Do not report “fully implemented,” “secure,” “private,” “verified” or “production ready” based only on source inspection. State exactly which tests ran and what remains unverified.

---

## 22. Phased implementation and hard stops

This is the intended full product. Phases organize dependency order; they are not permission to stop at a stripped-down MVP or to skip later gates.

### Phase 0 — Specification and benchmark freeze

- Inventory the repository, current architecture, manifests and installed skills.
- Map existing code to requirements and identify true dead code separately from required-but-unfinished code.
- Implement ADR-010 through ADR-013 as binding decisions; inventory the repository and remove contradictory Tauri/Wry plus external-Chromium assumptions from the production architecture.
- Prove CEF native view composition, local app scheme isolation, renderer message boundary, focus/input and lifecycle on macOS Apple silicon and Windows x64 before building product screens.
- Freeze action/state schemas, token roles, platform constraints and benchmark definitions.
- Recalculate and automate contrast tests.
- Define at least 600 unsafe/consequential and 600 safe/eligible labeled security examples plus a separate adversarial suite and frozen held-out split.
- Record all remaining implementation limitations with evidence and a binding safe behavior.
- Create the requirement-to-evidence matrix before implementation and keep it updated in every phase.

### Phase 1 — Browser foundation and vertical slice

- Build the CEF native host, React/TypeScript first-party UI views, permanent Agent panel and actual remote CEF browser views.
- Verify local new-tab first launch, URL entry, navigation, tabs, focus, resizing and manual browsing without a model.
- Ensure the displayed page and action target are the same active CEF browser instance and no production remote-debugging endpoint is exposed.
- Add a minimal end-to-end supported task with real perception, typed action, policy, execution and verification.

### Phase 2 — Reliable perception and deterministic execution

- Implement CEF browser/frame/renderer lifecycle handling, accessible/DOM observations, semantic state graph, hit-target freshness checks, epoch invalidation, bounded waits and cancellation.
- Exercise shadow DOM, OOPIF and stale-target cases.
- Add real execution telemetry to the Agent panel without exposing raw engineering internals by default.

### Phase 3 — Policy, confirmations and secure data

- Implement action classes, policy checks, origin boundaries, structured confirmations, secure-fill isolation, audit chain and hard-stop semantics.
- Prove that model output and page content cannot authorize an action.
- Prove that consequential writes require the required human gate.

### Phase 4 — Independent verification and recovery

- Build task-specific pre/postconditions and evidence-backed outcomes.
- Implement safe pause, takeover, resume, crash recovery and unknown-result handling.
- Test side-effect ambiguity after a crash so writes are not replayed blindly.

### Phase 5 — Local inference and model/resource evaluation

- Benchmark the candidate models/runtimes on reference hardware.
- Select and freeze the model only after the defined evaluation.
- Enforce structured output and latency/memory limits.
- Follow Section 16's contingency if no candidate passes.

### Phase 6 — Skills, persistence and long workflows

- Implement validated reusable skill versions, current-page preconditions, persistent task metadata and explicit resume protocols.
- Evaluate interruptions, page drift, authentication boundaries and longer task recovery.

### Phase 7 — Adversarial, cross-platform and resource gates

- Test prompt injection, malicious page instructions, intent collision, origin changes, stale target attacks, secret exfiltration, network/navigation traps, authorization mismatches, corrupted state and resource exhaustion.
- Run the macOS Apple-silicon and Windows x64 security/browser suites.
- Verify production builds contain no CDP endpoint, local HTTP control server or privileged bridge in remote page contexts.
- Meet the labeled-corpus thresholds and pass all critical invariants or block release and remediate.

### Phase 8 — Packaging and lifecycle

- Produce real macOS/Windows installers, update/integrity procedures, data migration, profile lifecycle, uninstall behavior and privacy documentation.
- Complete release validation, reproducibility and signature/notarization steps where configured.

### Hard-stop rules

[BA-STOP-001] Stop the build/release gate if any of these occur:

- browser page is fake, detached, blank, overlaid by an input-swallowing shell, or not the actual managed CEF view;
- the UI reports success without verification evidence;
- an unknown action bypasses policy;
- secret material enters model prompts, frontend state or ordinary logs;
- a high-impact action proceeds without its required authorization;
- a forbidden Chromium flag, production CDP endpoint, unauthenticated local control service or privileged bridge to remote website content is introduced;
- required security checks are omitted or replaced with self-reported prose;
- a claimed local-only feature silently uses remote inference;
- a false state, fake data source or inert primary control remains in the default UX;
- a mandatory accessibility, contrast or permanent-Agent constraint fails.

On a hard stop, preserve safe state, write a redacted audit record, provide a truthful status and require explicit intervention or a documented safe resume path.

---

## 23. Final acceptance statement

Browser-Agent is accepted only when it feels like a carefully crafted, familiar desktop browser that can perform real, policy-governed work—not a dashboard, not a chatbot, and not an animation showcase.

Its distinctive interface is built from:

- a real managed CEF/Chromium webpage in the same native view hierarchy as the app chrome;
- familiar browser controls and tabs;
- a permanently visible but compact Agent working region;
- serif-first local typography;
- the immutable seven-color earthy palette with correct semantic pairings;
- selective small-radius geometry;
- minimal, explicitly restricted app-owned icons;
- flat opaque surfaces and purposeful borders;
- immediate feedback with almost no motion;
- responsive layout and lazy loading that preserve immediate usability;
- accurate states and honest verification;
- robust policy, privacy, secure-fill, audit and recovery behavior;
- no production browser-control network listener and isolated app/UI versus website contexts.

All architectural/scope questions represented in Appendices F and G are binding decisions for this revision; implementation agents must not reopen them by preference or silently broaden scope. Only a reviewed amendment under Section 0.7 may change them. Every requirement remains unverified until the proof artifact named in Appendix D exists.

The premium quality bar is not “does this look like a modern AI app?” It is:

> **Can a person understand the browser immediately, keep the webpage and Agent in view, use every control confidently, and trust the system's account of what actually happened?**

---

## Appendix A — Official Apple guidance and external standards

These sources guide the principles, not a requirement to imitate Apple's current interface treatment. Review the latest pages before a major design revision because platform guidance can change.

### Apple Human Interface Guidelines

- [Design principles](https://developer.apple.com/design/human-interface-guidelines/design-principles/) — purpose, agency, flexibility, simplicity, craft, and the distinction between delight and decoration.
- [Designing for macOS](https://developer.apple.com/design/human-interface-guidelines/designing-for-macos/) — desktop productivity, resizing, full-screen behavior, menu bar, keyboard use and comfortable information density.
- [Branding](https://developer.apple.com/design/human-interface-guidelines/branding) — judicious accent-color use, custom-font legibility, familiar components and content-first branding.
- [Layout](https://developer.apple.com/design/human-interface-guidelines/layout) — hierarchy, alignment and adaptive composition.
- [Toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars) — purposeful action grouping and avoiding toolbar overcrowding.
- [Menus](https://developer.apple.com/design/human-interface-guidelines/menus) — familiar command behavior, text labels and sparing, meaningful menu icons.
- [Motion](https://developer.apple.com/design/human-interface-guidelines/motion) — motion should help users understand changes and support platform interaction patterns.
- [Typography](https://developer.apple.com/design/human-interface-guidelines/typography) — type legibility, size and accessibility considerations.
- [Color](https://developer.apple.com/design/human-interface-guidelines/color) — purposeful color usage and contextual meaning.
- [Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility) — inclusive controls, readable content and input support.
- [Materials](https://developer.apple.com/design/human-interface-guidelines/materials) — Apple's treatment of materials and visual hierarchy; Browser-Agent deliberately chooses opaque surfaces instead of Liquid Glass.

### Browser security, URL display and localization references

- [Chromium URL display guidelines](https://github.com/chromium/chromium/blob/main/docs/security/url_display_guidelines/url_display_guidelines.md) — canonical URL formatting, spoofing-sensitive font display, bidi controls and IDN behavior. Use the URL/IDN behavior of the pinned engine; do not implement a separate home-grown canonicalizer.
- [Chromium IDN handling](https://chromium.googlesource.com/chromium/src/%2B/master/docs/idn.md) — script-mixing, punycode and Unicode confusable decisions.
- [Apple Human Interface Guidelines: Right to left](https://developer.apple.com/design/human-interface-guidelines/right-to-left) — mirroring layouts and respecting script direction.
- [W3C WCAG 2.2](https://www.w3.org/TR/WCAG22/) — keyboard/focus/status messages and accessibility criteria.

### Web accessibility and motion references

- [WCAG 2.2](https://www.w3.org/TR/WCAG22/) — text and non-text contrast, keyboard operation, focus visibility, target size and other applicable requirements.
- [MDN: `prefers-reduced-motion`](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-motion) — implementation reference for respecting the user's reduced-motion preference.
- [Nielsen Norman Group: Aesthetic and minimalist design](https://www.nngroup.com/articles/ten-usability-heuristics/) — irrelevant information competes with important content.
- [GOV.UK Design System: Icons](https://design-system.service.gov.uk/styles/icons/) — practical guidance on limiting unnecessary icons and preserving label clarity.
- [W3C Internationalization: Structural markup and right-to-left text](https://www.w3.org/International/questions/qa-html-dir) — directionality markup, bidi behavior and CSS logical properties for localization-ready interfaces.
- [CEF JavaScript integration: Extensions](https://github.com/chromiumembedded/cef/blob/master/docs/javascript_integration.md) — documents `CefRegisterExtension`, the legacy custom V8-extension mechanism; its M154 deprecation note does not by itself mean Chrome-style extension packages are unsupported.
- [CEF Request Context API](https://github.com/chromiumembedded/cef/blob/master/include/cef_request_context.h) — documents `CefRequestContext::LoadExtension`, extension accessors, and request-context cache-path behavior. Verify the exact extension API support list and empty-cache-path behavior against the pinned CEF headers/build; test that Private Window browsing data does not leak into the persistent profile.

### Evidence caveat

This specification uses Apple's published guidelines as a design-quality reference. It does not claim Apple certifies or endorses Browser-Agent, and it does not claim that following HIG alone guarantees a premium product. The product's serif-first typography, narrow icon whitelist, permanent Agent panel and selective corner rules are deliberate Browser-Agent requirements, not universal Apple mandates.

---

## Appendix B — Architecture research and binding rationale

Research review date: 2026-10-13. Links below support technology decisions; they do not prove that Browser-Agent itself is implemented or tested.

### Embedded browser host

- [Chromium Embedded Framework: General Usage](https://github.com/chromiumembedded/cef/blob/master/docs/general_usage.md) — CEF is explicitly designed for embedding an HTML5 browser control or web-based UI in native applications; documents its multiprocess model, native browser handles, off-screen rendering, IPC, and optional DevTools remote debugging.
- [CEF Tutorial](https://chromiumembedded.github.io/cef/tutorial) — documents CEF process architecture and browser styles/views. CEF on macOS requires a separate subprocess executable/app-bundle arrangement; packaging must account for this.
- [Tauri Architecture](https://github.com/tauri-apps/tauri/blob/dev/ARCHITECTURE.md) — Tauri hosts HTML through an operating-system WebView. The decision to remove Tauri/Wry from the production rendering path is specific to Browser-Agent's requirement to combine its chrome and a true managed Chromium webpage in one native view hierarchy; it is not a general claim that Tauri is unsuitable for ordinary applications.
- [Tauri sidecar documentation](https://v2.tauri.app/develop/sidecar/) — sidecars are supported, but a supported sidecar mechanism alone does not solve embedding or input/focus synchronization for a separate browser window.

### CDP and isolated browser profiles

- [Chrome for Developers: Changes to remote debugging switches](https://developer.chrome.com/blog/remote-debugging-port?hl=en) — since Chrome 136, `--remote-debugging-port` and `--remote-debugging-pipe` require a non-default `--user-data-dir`; Chrome recommends isolating automation profiles and points automation users to Chrome for Testing.
- [chromiumoxide PR #325](https://github.com/mattsse/chromiumoxide/pull/325) — as checked on 2026-10-13, the pipe-support PR is marked Draft; the author says the changes were verified on Linux but not tested on Windows. It is not a sound production dependency for a cross-platform release gate.
- [chromiumoxide PR #331](https://github.com/mattsse/chromiumoxide/pull/331) — as checked on 2026-10-13, session-aware cross-origin iframe support remains an open pull request. This supports avoiding an assumption that these capabilities already exist in a stable, cross-platform production dependency.

### Local inference

- [llama.cpp](https://github.com/ggml-org/llama.cpp) — documents Apple silicon/Metal support, CPU backends and additional hardware backends. Browser-Agent will use one benchmarked production engine rather than maintaining MLX and llama.cpp as parallel runtimes.
- [MLX](https://github.com/ml-explore/mlx) — an array framework designed for Apple silicon. It remains a possible research/benchmark reference, but is not the chosen cross-platform production runtime under ADR-011.

### Security and accessibility

- [OWASP Top 10 for LLM Applications](https://owasp.org/www-project-top-10-for-large-language-model-applications/) — includes prompt injection, sensitive information disclosure, insecure plugin design, excessive agency and unbounded consumption risks relevant to a browser agent.
- [WCAG 2.2: Target Size (Minimum)](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum) — the Level AA criterion uses a 24 by 24 CSS-pixel target-size rule with documented exceptions. Apply the criterion to first-party UI, with spacing/equivalent-function exceptions documented rather than assumed.

### Evidence boundary

External documentation establishes that an API/framework exists or a risk is recognized. It does not establish that the pinned CEF version, Rust/C++ bridge, target hardware, layout, model or policy passes Browser-Agent's requirements. Those claims require repository-specific tests and release evidence as defined in Sections 20–22.

---

## Appendix C — Engineering standards, CI/CD and security references

Research review date: 2026-10-13. These sources ground the repository rules in current upstream documentation. They establish tool capabilities and recommended usage; they do not prove that Browser-Agent's own pipeline is configured or passing.

### Rust toolchain, formatting, testing and lints

- [The Cargo Book: Continuous Integration](https://doc.rust-lang.org/cargo/guide/continuous-integration.html) — Cargo CI patterns, workspace testing, MSRV checks and warning handling.
- [Rust 1.99.0 release announcement](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/) — the initial pinned Rust baseline selected for this specification.
- [Clippy usage](https://doc.rust-lang.org/stable/clippy/usage.html) — run Clippy in CI and promote relevant warnings to errors.
- [Rustfmt](https://rust-lang.github.io/rustfmt/) — checked-in formatter configuration and the need to match the configuration to the selected toolchain.
- [The Rustonomicon: How Safe and Unsafe Interact](https://doc.rust-lang.org/stable/nomicon/safe-unsafe-meaning.html) — the purpose of the unsafe boundary and use of `forbid(unsafe_code)` where possible.
- [Rust compiler lint levels](https://doc.rust-lang.org/rustc/lints/levels.html) — distinctions between warn, deny and forbid.
- [RustSec cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit) — auditing `Cargo.lock` against the RustSec Advisory Database.
- [cargo-deny](https://embarkstudios.github.io/cargo-deny/checks/index.html) — advisories, license, dependency bans and source checks.

### TypeScript, React and formatting

- [TypeScript TSConfig reference](https://www.typescriptlang.org/tsconfig/) — strict compiler options and type-safety controls.
- [typescript-eslint getting started](https://typescript-eslint.io/getting-started/) — ESLint flat config, recommended TypeScript rules and typed linting.
- [Prettier configuration](https://prettier.io/docs/configuration) and [Prettier CLI](https://prettier.io/docs/cli) — reproducible project-local configuration and `--check` as a CI gate.
- [React common DOM components](https://react.dev/reference/react-dom/components/common) — the XSS risks of `dangerouslySetInnerHTML` with untrusted content.
- [Node.js release schedule](https://nodejs.org/en/about/previous-releases) — selection of a supported LTS line rather than an unsupported or floating runtime.
- [Node.js 24.21.0 LTS release](https://nodejs.org/en/blog/release/v24.21.0) — the frontend Node baseline selected during the 2026-10-13 research pass.

### GitHub Actions, dependency and artifact security

- [GitHub Actions secure use reference](https://docs.github.com/en/actions/reference/security/secure-use) — full-length commit SHA pinning and workflow security practices.
- [GitHub protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches) — requiring passing status checks before merge.
- [GitHub dependency review](https://docs.github.com/en/code-security/concepts/supply-chain-security/dependency-review) — inspect newly added or updated dependencies in pull requests. Availability can depend on repository visibility and plan.
- [GitHub CodeQL supported languages](https://codeql.github.com/docs/codeql-overview/supported-languages/) — current language coverage includes C/C++, Rust, JavaScript/TypeScript and GitHub Actions. A compiled-language build must be configured correctly for useful analysis.
- [OSV-Scanner source scanning](https://github.com/google/osv-scanner/blob/main/docs/scan-source.md) — scanning source repositories, lockfiles and SBOMs for known vulnerabilities.
- [Gitleaks Action](https://github.com/gitleaks/gitleaks-action) — automated secret detection; currently use a maintained Node 24-compatible release and pin it to a reviewed full commit SHA in the production workflow.
- [GitHub artifact attestations](https://docs.github.com/en/actions/concepts/security/artifact-attestations) — signed provenance records connecting build artifacts to repositories, commits, workflows and environments.
- [Clang-Tidy](https://clang.llvm.org/extra/clang-tidy/) — native C/C++ static analysis in addition to formatting and compiler diagnostics.

### Interpretation rule

Use upstream documentation for tool configuration and security updates. Verify the exact action commit SHA, tool version, operating-system runner image and license at implementation time. Do not paste mutable tags or claim checks are passing merely because a workflow file was created; required jobs must execute and their artifacts must be reviewed.



---

## Appendix D — Normative requirement registry and evidence matrix

This registry is generated from `requirements/registry.yaml`, the canonical machine-readable source. The inline ID placed at the relevant section's normative text is the authoritative requirement statement; this table binds it to expected verification evidence. IDs are permanent and must not be renumbered. Phase 0 cannot close until all release-critical requirements are represented; unlike earlier drafts, do not defer registry completion to an unspecified later pass.

| Requirement ID | Source | Required proof artifact / test family | Release gate |
|---|---|---|---|
| `BA-DESIGN-001` | 0.3 | Design review checklist plus screenshots of all Section 21.1 states; run regardless of Impeccable availability | Required |
| `BA-CEF-001` | 11.1 | CEF lifecycle/sandboxing, process shutdown/restart and production-debugging-disabled tests | Required |
| `BA-POLICY-001` | 13.1 | Tests proving model/page content cannot authorize actions and executor requires Rust authorization | Critical safety gate |
| `BA-LOCAL-001` | 14.1 | Network tests proving no cloud inference requirement or secret/content relay | Critical privacy gate |
| `BA-AUDIT-001` | 15.1 | Schema/canonicalization, key-access, rotation, continuity, persistence-failure and tamper tests | Critical safety gate |
| `BA-RESOURCE-001` | 15.2 | Timeout/backpressure/hard-stop and no-policy-bypass under memory pressure tests | Required |
| `BA-SEC-001` | 15.4 | Frozen unsafe/safe corpora, invariant tests and confidence-interval report | Critical safety gate |
| `BA-RECOVERY-001` | 15.6 | Crash/ambiguous-write recovery tests proving no blind replay | Critical safety gate |
| `BA-MODEL-001` | 16.3 | Reproducible local inference memory/latency/success/thermal report | Required |
| `BA-A11Y-001` | 18.1 | Keyboard/screen reader/focus/target-size tests on supported platforms | Required |
| `BA-CI-001` | 20.9 | Required named status-check report for Rust, UI, native CEF, security and contract drift | Merge gate |
| `BA-SUPPLY-001` | 20.10 | Dependency, secrets, native binary, license, SBOM and workflow-security reports | Required |
| `BA-RELEASE-001` | 20.11 | Signed platform packages, provenance, SBOM, update/rollback validation | Release gate |
| `BA-DONE-001` | 21.5 | Release checklist linking actual evidence for every release-critical ID | Release gate |
| `BA-STOP-001` | 22 | Hard-stop test cases and release-gate behavior | Critical safety gate |
| `BA-ARCH-002` | ADR-010 | CEF feasibility report, real-window test recording, macOS and Windows packaged smoke tests, decision status | Block dependent implementation if not passed |
| `BA-TRACE-001` | 0.6 | Stable ID parser/check plus registry/matrix consistency test | Required |
| `BA-TRACE-002` | 20.4 | CI contract-drift report and evidence matrix | Required |
| `BA-UNDO-001` | 2.7 | Unit/integration tests for supported undo, unsupported inverse, stale state, Agent pause and redo reauthorization | Required |
| `BA-WIN-001` | 2.14 | Multi-window open/close/focus tests; global single-task concurrency test; owner-window closure recovery test | Required |
| `BA-WIN-002` | 12.2 | Cross-window runtime test proving exactly one active task process-wide and rejection of silent queue/concurrency | Critical safety gate |
| `BA-EXT-001` | 2.15 | Product/UI test proving no extension installation surface or arbitrary extension execution path ships | Required scope assertion |
| `BA-PRIV-001` | 2.16 | Cross-context cookie/site-storage isolation plus disk-artifact inspection after private-window close | Required |
| `BA-I18N-001` | 3.6 | Pseudo-localization, RTL, CJK, Devanagari, Unicode/IME and font-coverage reports | Required |
| `BA-LAYOUT-001` | 3.4 | Geometry assertion at 900×640 client area; screenshots/tests for 360 px browser + 200 px Agent minima | Required |
| `BA-STATE-001` | 2.9.1 | Table-driven state-to-copy tests over every task state and wait reason | Required |
| `BA-STATE-002` | 12.2 | Schema validation rejecting missing/unknown wait_reason values | Required |
| `BA-BRIDGE-001` | 9.1, 20.7 | Bundle/eager-load check and bridge allow-list/command-boundary tests | Required |
| `BA-BROWSER-001` | 11.2 | Native CEF view, input/focus/IME/resizing and multi-window smoke tests on both OS targets | Required |
| `BA-REV-001` | 13.2 | Positive and negative reversibility fixtures; compensation precondition/postcondition and residual-effect tests | Critical safety gate |
| `BA-PASS-001` | 13.6 | Manual vault origin/frame tests, explicit-save flow tests and proof credentials never reach model/log/UI state | Required |
| `BA-TELE-001` | 14.2 | Network-capture tests proving no background analytics/crash upload; export preview/redaction tests | Required privacy gate |
| `BA-LOOP-001` | 15.3 | Frozen 1,000-benign/300-adversarial livelock calibration and held-out reports | Required |
| `BA-VERIFY-001` | 15.5 | Table-driven lifecycle/outcome cross-product tests including blocked-before-dispatch and crash ambiguity | Critical safety gate |

| `BA-AMEND-001` | 0.7 | Reviewed amendment with rationale, impacted IDs, tests and ADR linkage | Governance gate |
| `BA-REQ-LINT-001` | 0.7 | Registry generator/linter: unique IDs, source consistency and 100% critical coverage | CI gate |
| `BA-THREAT-001` | 0.7 | Threat model mapped to mitigations and negative tests | Security gate |
| `BA-REVIEW-001` | 0.7 | Named release-owner design/security checklist and required screenshots | Release gate |
| `BA-URL-001` | 2.5 | HTTP/certificate/mixed-content/IDN/bidi/canonical display and spoofing test suite | Critical safety gate |
| `BA-TABSTATE-001` | 2.4–2.5 | Real CEF loading/crash/audio/download status tests; no prohibited icons/animations | Required |
| `BA-INPUT-001` | 2.5 | Packet capture proves suggestions/spellcheck disabled by default; opt-in/private-mode tests | Privacy gate |
| `BA-READY-001` | 2.9.1 | Table-driven model readiness state/copy/action tests | Required |
| `BA-COMPOSER-001` | 2.9.2 | Unicode code-point cap/warn-no-truncate, local draft retention and private-mode tests | Required |
| `BA-TRANSPARENCY-001` | 2.9.2, 13.3 | Disclosure-content test plus blocked-state policy-manifest drill-in test | Required |
| `BA-BUDGET-001` | 2.9.2 | Visible budget accounting, extension authorization and budget-exhaustion UI | Required |
| `BA-APPROVAL-001` | 2.12 | Five-minute TTL, one-pending approval, expiry/invalidation and no stale approval execution | Critical safety gate |
| `BA-INTERRUPT-001` | 2.12 | In-flight action stop boundary, sleep/wake, no-focus-steal and ambiguous-side-effect tests | Critical safety gate |
| `BA-WIN-003` | 2.14, 12.2 | Same-window second-task disabled; task-tab ownership/tear-off/popup boundary tests | Critical safety gate |
| `BA-PRIV-002` | 2.16 | Private new-tab, autocomplete/form-fill disabled and no-post-close-artifact tests | Privacy gate |
| `BA-TYPE-001` | 3.1 | Tabular numeral, URL confusable and typeface rendering tests | Required |
| `BA-LAYOUT-002` | 3.4 | Stacked panel clamp formula and non-resizable geometry assertions | Required |
| `BA-LAYOUT-003` | 3.4 | Toolbar minimum-width and localization/pseudo-localization overflow tests | Required |
| `BA-THEME-001` | 3.4, 6 | OS light/dark selection and full screenshot/contrast matrix in both themes | Required |
| `BA-COPY-001` | 4.4 | UI string-resource lint with reviewed allowlist | CI gate |
| `BA-FIND-001` | 2.8, 4.4 | Native find-in-page, keyboard shortcuts and screen-reader tests | Required |
| `BA-SOUND-001` | 4.4 | UI review/asset scan proving no app-owned sounds | Required |
| `BA-BRIDGE-002` | 17.1 | Per-message origin validation, C ABI fuzz/sanitizer and bridge audit checklist | Critical safety gate |
| `BA-NORMALIZE-001` | 12.2 | Two-round clarification limit and unsafe-ambiguity wait tests | Required |
| `BA-STUCK-001` | 12.2 | Repeated no-progress action digest test; no third identical dispatch | Critical safety gate |
| `BA-TABSCOPE-001` | 12.2, 2.14 | Task-owned tab set, cross-tab grant, cross-window denial and tear-off test | Critical safety gate |
| `BA-OBSERVATION-001` | 12.5 | Node/depth/token/time budget tests; serialization order and no-raw-HTML checks | Required |
| `BA-RETRY-001` | 12.5 | Action-specific retry matrix and ambiguous-write non-retry tests | Critical safety gate |
| `BA-EGRESS-001` | 13.3 | Typed-data egress classification, vault-value detection and destination-change tests | Critical safety gate |
| `BA-ALLOWLIST-001` | 13.3 | Task-scoped origin allowlist enforcement and honest subresource limitation disclosure | Required |
| `BA-CLIPBOARD-001` | 13.3, 17.1 | No clipboard access without single-use approval | Privacy gate |
| `BA-UNMASK-001` | 13.3 | Reveal action requires user gesture and records non-secret audit metadata | Required |
| `BA-GRANT-001` | 13.3, 17.1 | Agent cannot grant permissions; hostname typed-confirmation and certificate bypass restrictions | Critical safety gate |
| `BA-POLICY-PERF-001` | 13.4 | Policy microbenchmarks and p95 evidence without cross-epoch verdict reuse | Required |
| `BA-AUDIT-002` | 15.1 | UTC/monotonic clock tests, rotation, retained-range wording and fail-closed audit-write failure | Critical safety gate |
| `BA-TASK-BUDGET-001` | 15.2 | Action/nav/token/time ceiling tests; single bounded extension | Required |
| `BA-MEMORY-001` | 15.2, 16.3 | OS memory-pressure and app-process ceilings; park/warm behavior by hardware class | Required |
| `BA-STARTUP-001` | 15.2 | p95 window/address-field start SLO report, model off critical path | Required |
| `BA-MODEL-PROFILE-001` | 16.2 | Multiple installed models / one resident, profile compatibility and no mid-task swap tests | Required |
| `BA-MODEL-REPORT-001` | 16.2 | Optional local calibration/integrity check; report provenance and replay-only safety tests | Required |
| `BA-SKILLS-001` | 17 | Skill inspect/save/disable/delete/revoke/provenance and immutable-version tests | Required |
| `BA-HISTORY-001` | 17 | 90-day local history, similar-task prefill, private exclusion and redacted report export | Privacy gate |
| `BA-DOWNLOAD-001` | 17.1 | Per-file approval, path/size checks, quarantine, origin provenance and no-auto-open tests | Critical safety gate |
| `BA-A11Y-002` | 18.1 | Cancel-default focus, coalesced live regions, target text, focus restoration and screen readers | Required |
| `BA-ERROR-COPY-001` | 19 | Worked error states and copy contract tests | Required |
| `BA-FAULT-001` | 20 | Fault-injection matrix for audit disk-full, renderer/network/model failures | Critical safety gate |
| `BA-REQ-LINT-002` | 20, Appendix D | CI validates generated registry, copy lint and bridge API drift | CI gate |
| `BA-UPDATE-001` | 20.11 | Signed metadata/hash verification, staged update/rollback and local-only crash-report policy | Release gate |
| `BA-DATA-001` | 14.3 | Forget-site/clear-all category preview and deletion behavior | Privacy gate |
| `BA-VAULT-001` | 14.3 | OS secure storage, 5-minute auto-lock, lock-on-session/quit, no plaintext fallback | Critical safety gate |
| `BA-NOTES-001` | 17 | Local notes source URLs/epochs/excerpts, search/export/delete and private exclusion | Required |
| `BA-DATA-MAP-001` | Appendix E | Artifact inventory, sensitivity, retention and deletion-path audit | Privacy gate |
| `BA-FEATURE-001` | Appendix F | Supported/non-goal/deferred feature register and release/UI conformance tests | Scope gate |

| `BA-SESSION-001` | 2.4 | Session restore opt-in; no private/task/action/secret restoration | Required |
| `BA-ZOOM-001` | 2.4 | Normal-profile per-origin zoom and private-context cleanup tests | Required |
| `BA-MODEL-ISOLATION-001` | 16.2 | Fresh context/KV cache per task; no cross-task prompt-context leakage | Critical privacy gate |
### D.1 Required evidence-matrix fields

The implementation repository must maintain a machine-readable CSV, JSON or YAML matrix with at least these fields: `requirement_id`, `source_section`, `summary`, `priority`, `owner_module`, `test_ids`, `evidence_artifacts`, `status`, `last_verified_commit`, `last_verified_at`, `platform`, `approved_deviation_id`, and `notes`. Do not store secrets, full browsing traces or raw personal data in the matrix.

### D.2 Coverage policy

Phase 0 cannot be marked complete until all release-critical requirements in the document are enumerated by stable ID, the registry and matrix agree, and every requirement has an owner and a test/evidence plan. A requirement without implementation/evidence stays unverified; absence of a test is not a passing test. When new release-critical requirements are added, assign new IDs and add rows in the same change.

## Appendix E — On-disk data map, retention and deletion

[BA-DATA-MAP-001] The data map below is normative. “Local only” means Browser-Agent does not transmit the artifact to a Browser-Agent service; normal browsing requests, explicitly selected search providers, explicit update checks and user-approved site submissions still go to their declared remote destination.

| Artifact | Location/boundary | Sensitivity | Default retention | User control and deletion behavior |
|---|---|---|---|---|
| Normal CEF profile (cookies, local storage, cache, history) | App-owned persistent profile directory with user-only filesystem permissions | High | Until user clears it; history target 180 days | Clear browsing data / Forget this site; never merge with user's ordinary Chrome profile |
| Private CEF context | Ephemeral per-private-window context; never persistent profile path | High | Window lifetime only | Destroy context on close; inspect for residual site-data artifacts in tests; explicit downloads/bookmarks can outlive it after notice |
| Credentials/vault | OS Keychain / Windows Credential Manager or supported DPAPI-protected storage | Critical | Until user deletes; encrypted/OS-protected at rest | Explicit unlock, save, export and delete; 5-minute inactivity auto-lock; no plaintext fallback |
| Model weights and manifest | App-owned user data `models/` directory | Medium | Until user removes or update/revocation policy requires removal | Verify integrity, switch approved model, remove artifact; never commit to Git |
| Model runner executable/runtime | Signed packaged application resources | High integrity | App version lifecycle | Verified in release; update only from signed manifests and reviewed package pipeline |
| Agent task history and reports | App-local database | High | 90 days default | Clear history, export preview/redact, run-similar prefill; private tasks excluded |
| Audit events | App-local chained HMAC segments | High integrity | 90 days default, segmented daily or 10 MiB | User can purge old ranges; UI reports integrity only over retained range after pruning |
| Research notes | App-local data store | User-controlled content | Until deleted | Exact-match local search, edit, export and delete; each saved excerpt has origin and capture time |
| Skills | App-local immutable version store | Medium | Until user deletes/revokes | Inspect provenance, disable, revoke, delete; no automatic sync |
| Downloads | User-chosen destination; agent downloads staged in quarantine | Variable/high | Until user deletes | Downloads view shows origin/provenance; never auto-open; does not disappear on Private Window close if explicitly saved |
| Temporary observations/checkpoints | Encrypted or permission-restricted app-local ephemeral working storage; redact before persistence | High | Current task/recovery window; cleanup after terminal state plus retention needed for safe recovery | Clear task history/data; do not persist raw private-window content |
| Diagnostic logs/replay hashes | App-local, redacted | Medium/high | 30 days default; no raw prompt/page dumps | User-initiated redacted export; never auto-upload |
| Settings/search provider choice | App-local settings | Low | Until user resets | Clear settings; no cloud sync |

`Forget this site` removes normal-profile cookies/site storage/cache/history, per-origin permission grants, saved notes and task-history references for the selected origin after a preview. It asks separately before deleting matching vault entries or downloaded files. `Clear all Browser-Agent data` presents a category checklist and does not silently delete vault entries or user downloads. `Export Browser-Agent data` uses a separate category checklist with preview/redaction; default export contains only selected bookmarks, normal browsing history, settings, local notes, skills and redacted Agent task/audit summaries. It never includes private-window content, cookies, live sessions, vault secrets, raw page DOM, raw model prompts or model weights by default. Any higher-risk category requires a separate explicit decision and must be excluded if it cannot be safely redacted. All deletion copy must identify the exact categories affected and must not promise secure erasure from SSDs, OS backups, filesystem snapshots or remote websites.

## Appendix F — Supported features, non-goals and deferred capabilities

[BA-FEATURE-001] The following register defines the full-product feature envelope. A feature cannot appear in the UI merely because it is typical of another browser; it must be implemented and tested. “Deferred” means it is not shipped until the stated trigger and a scoped ADR/test plan are accepted. “Non-goal” means not on the current roadmap without a deliberate scope revision.

| Capability | Decision | Required behavior or revisit trigger |
|---|---|---|
| Find in Page | Supported | Native CEF search API; Command + F / Control + F; keyboard/screen-reader tested |
| Textual tab loading/crash/audio states | Supported | Text only; no new app-owned icons or ornamental loading animation |
| Download progress | Supported | Textual percentage/state from real CEF events; per-Agent-download approval and quarantine |
| Tab reorder | Supported | In-window reorder; never changes active task authorization without invalidation |
| Tab tear-off/new window | Supported when no Agent task is active | Block tear-off for task-owned tabs while any Agent task is active |
| Session restore | Supported for normal browsing | Opt-in user setting; restore normal tab URLs only; never restore private windows, task contexts, approvals, model prompts, or auto-resume Agent tasks; see BA-SESSION-001 |
| Per-site zoom memory | Supported | Local normal-profile storage; private zoom state discarded on close; see BA-ZOOM-001 |
| Browser extensions and extension stores | Non-goal | Revisit only through a dedicated threat model, sandbox/permission architecture and ADR |
| Private-window Agent execution | Non-goal | Agent, task history, notes and vault autofill disabled in private windows |
| Multiple normal profiles | Deferred | Revisit after stable profile lifecycle, independent vault/origin binding and cross-profile tests; current product has one normal persistent profile plus private contexts |
| Browser sync/account login | Non-goal | No cloud sync or account requirement; revision requires new privacy/security ADR |
| Built-in translation | Deferred | Revisit only with a locally executed or explicitly user-approved engine, text provenance and privacy review; no cloud translation fallback |
| Reader mode | Deferred | Revisit after accessibility, source attribution and webpage transformation tests |
| First-party spellcheck | Deferred | May ship only using a locally bundled dictionary, no cloud requests, language/license/IME tests; off in initial release |
| Search-provider suggestions | Optional user opt-in | Off by default; explicit disclosure that query fragments are sent to selected provider; unavailable in private windows |
| More than one concurrent Agent task | Deferred | Review only after a fresh ADR once a 16 GB+ reference class shows ≥4 GB measured headroom under the simultaneous workload, no sustained system memory pressure, correct cross-task policy isolation, and both tasks independently meet latency/safety gates. This is a revisit trigger, not a delivery promise. |
| Remote telemetry/crash uploads | Non-goal | Any proposal requires new privacy ADR, off-by-default explicit opt-in, payload preview, retention and withdrawal path |
| App-owned UI sounds | Non-goal | No sound-only signaling; websites control website audio under permissions |

## Appendix G — Locked decision register for former open questions

The following choices close the open questions raised during review; they are binding without requiring a separate user decision. Any apparent conflict is resolved by the higher-level precedence in Section 0.1 and the feature register in Appendix F. No “open question” placeholder remains in this revision.

1. **Security states without icons:** scheme/host are always textual; HTTP is labelled `Not secure — HTTP`; certificate errors are explicit; mixed-content warning appears only when reliable CEF evidence exists. Loading/crash/audio/download states are textual as defined in BA-TABSTATE-001.
2. **Tab audio/download progress:** supported through text status, not new icons.
3. **Toolbar width:** English first release uses the 960 px breakpoint and measured minimum controls in BA-LAYOUT-003; later locale strings may trigger earlier stacking. Never truncate mid-word.
4. **Concurrency:** one Agent task per application process. Revisit only when the Appendix F evidence trigger is met.
5. **Serif numerals:** require tabular numerals from a licensed serif face; no sans-serif fallback.
6. **Second task in the owning window:** composer disabled with an explanatory message; no silent queue.
7. **Approval lifetime:** one pending approval, maximum 5 minutes, invalidated sooner on state drift.
8. **Pause/cancel/takeover:** stop new dispatch immediately; reconcile any in-flight action at its bounded boundary; unknown external side effects remain `UNKNOWN`.
9. **Task tabs and tear-off:** task-owned tabs stay in the owner window; tear-off is available only when no task is active.
10. **Private autofill:** both password and non-password form/address autofill disabled; private new tab is stateless.
11. **Task clarifications/stuck behavior:** at most two clarification rounds before proceeding only under safe stated assumptions; two no-progress repeats stop before a third identical action.
12. **Impeccable availability:** use it if installed; otherwise the same mandatory screenshot/design-review process applies.
13. **Undo compensation:** compensation uses the same explicit approval mechanism when the compensation itself needs approval.
14. **Telemetry:** zero off-device Browser-Agent analytics/usage telemetry/crash uploads. Explicit user-requested navigation/search/download/update is not mislabelled telemetry.
15. **Input assistance:** no network spellcheck or suggestions; search autocomplete is explicit opt-in and unavailable in private windows.
16. **First-launch networking:** local new-tab page; no unsolicited external page request.

