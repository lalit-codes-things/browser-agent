# DESIGN.md

## Browser-First Product UI/UX Specification

**Status:** New design direction
**Scope:** macOS + Windows desktop application
**Primary stack:** Tauri 2 + React/TypeScript + Rust
**Product shape:** Local-first autonomous web/computer agent with a browser-first interface

---

## 1. Product Direction

The application is no longer visually organized like a generic AI dashboard.

It should feel like a **browser that happens to have an autonomous agent built into it**.

The browser is the primary workspace.

The agent is a first-class capability inside that browser, not a separate product surface that happens to control a browser.

The primary mental model is:

```text
User opens app
    ↓
Browser shell
    ↓
Tabs / navigation / address bar
    ↓
Website viewport
    ↓
Agent can operate the page
    ↓
User can observe, intervene, approve, or take over
```

The product should not feel like:

```text
AI dashboard
+ embedded browser
```

It should feel like:

```text
Browser
+ autonomous control layer
```

---

## 2. Core UX Principle

The user should be able to understand the product without learning a new mental model.

A user familiar with Chrome, Edge, Safari, Arc, or another modern browser should immediately recognize:

- tabs;
- address/search bar;
- back/forward/reload;
- page viewport;
- downloads;
- bookmarks/favorites;
- browser settings;
- window controls.

The new capability is that the browser can act on the user's behalf.

The agent should therefore be visible at the moment it matters, but should not permanently dominate the interface.

---

## 3. Design Goals

The interface must be:

- browser-native in mental model;
- visually distinctive without becoming a toy;
- dense enough for real web work;
- readable for long sessions;
- keyboard accessible;
- mouse/trackpad friendly;
- optimized for desktop;
- suitable for autonomous operation and human takeover;
- explicit about agent state and consequential actions;
- free of decorative AI-dashboard clutter.

The UI should prioritize **information hierarchy, interaction clarity, and visible state** over ornament.

---

## 4. Design Personality

The visual language should retain the project's established preference for **custom typography, strong geometry, restrained neubrutalist influence, flat surfaces, and purposeful borders**, while adapting those ideas to a browser shell.

Do not create a stereotypical “AI product” interface.

Avoid:

- glowing purple/blue gradients;
- floating glass cards everywhere;
- fake 3D illustrations;
- excessive rounded cards;
- decorative AI sparkles;
- giant chat bubbles;
- meaningless icons;
- dashboard-style KPI cards;
- oversized marketing hero UI inside the actual app.

Use:

- strong typographic hierarchy;
- clear horizontal browser chrome;
- compact controls;
- deliberate borders;
- flat surfaces;
- subtle but visible state changes;
- motion only when it communicates action or state.

The browser viewport remains the dominant visual surface.

---

## 5. Application Shell

The app should have five primary visual layers:

```text
┌──────────────────────────────────────────────────────────────┐
│ Window chrome                                               │
├──────────────────────────────────────────────────────────────┤
│ Tab strip                                                   │
├──────────────────────────────────────────────────────────────┤
│ Navigation + omnibox + browser actions                      │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│                   Browser viewport                          │
│                                                              │
│          Agent overlay / Agent Cursor when active            │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│ Context/status strip when needed                             │
└──────────────────────────────────────────────────────────────┘
```

The exact macOS and Windows window chrome may differ according to native conventions, but the application-level browser structure remains consistent.

---

## 6. Tab Strip

The tab strip is the primary task container.

Each tab represents an independent browsing/task context.

A tab should expose only the information needed to identify it:

- site favicon when available;
- page title;
- loading/progress indicator when active;
- close control;
- selected state.

Do not put large AI labels into the tab title.

Agent activity may be shown through a subtle state indicator on the tab:

```text
idle
working
waiting for user
blocked
error
complete
```

The active tab must be visually unmistakable.

The new-tab control should remain in the familiar browser position.

---

## 7. Tab State

Tabs must distinguish browser state from agent state.

Browser state:

```text
loading
loaded
navigation pending
crashed
```

Agent state:

```text
idle
thinking
acting
waiting
confirmation required
paused
verification
completed
blocked
failed
```

Do not collapse these into one vague “loading” indicator.

---

## 8. Navigation Toolbar

The navigation toolbar should contain, in conventional order:

```text
Back
Forward
Reload/Stop
Address/Search bar
Browser actions
Agent control
```

The toolbar should not become overloaded with agent-specific controls.

The agent control should be the obvious entry point for autonomous behavior without displacing normal browser controls.

---

## 9. Omnibox / Address Bar

The address bar is the central command/input surface of the browser.

It must support normal browser behavior:

- URL navigation;
- search queries;
- paste;
- keyboard focus;
- selection/editing;
- navigation history behavior where supported.

The product may also allow natural-language task input from the address bar, but this must not turn the omnibox into a generic chatbot field.

When the user types a natural-language command, the UI should clearly distinguish:

```text
Navigate/search
```

from

```text
Agent task
```

without requiring a separate chat window.

---

## 10. Agent Entry Point

The agent should be accessible from the browser shell at all times without dominating the screen.

Preferred interaction model:

```text
Toolbar agent control
        ↓
Compact task composer
        ↓
Task becomes active in current tab or selected context
```

The composer is not a chat transcript.

It is a **task command surface**.

Example:

> Find the cheapest option from these results and open the best match.

Once submitted, the task becomes a visible browser activity rather than a message thread.

---

## 11. Agent Activity Surface

When the agent is active, show a compact activity surface connected to the browser viewport.

It should answer:

- What is the agent doing?
- What page is it operating on?
- What is it waiting for?
- Does it need permission?
- Did the action succeed?

Do not show internal chain-of-thought.

Do not expose hidden reasoning traces.

Display concise operational state such as:

```text
Finding the checkout button…
Filling the form…
Waiting for you…
Checking the result…
Done
```

---

## 12. Agent Cursor

The custom Agent Cursor is a central part of the browser experience.

It is a visual representation of the agent's actions, not an alternative execution system.

The real browser interaction continues to occur through the existing Rust/CDP execution path.

The cursor should visually:

- move toward the intended target;
- highlight the target when appropriate;
- show a click ripple;
- show typing activity without exposing secret characters;
- indicate drag operations;
- show waiting/paused state;
- show blocked/denied state;
- show completion/failure state.

The user's physical mouse cursor does not need to move.

---

## 13. Agent Cursor Visual Rules

The Agent Cursor should feel like a deliberate product identity element, not a fake operating-system cursor.

It may use:

- a custom shape;
- subtle motion trail;
- target ring;
- click ripple;
- short transition animations.

Do not make it overly large.

Do not make it look like a game HUD.

Do not animate continuously when the agent is idle.

Do not show cursor motion that does not correspond to a real browser action.

The cursor must stop or invalidate its target whenever the underlying browser state becomes stale.

---

## 14. Agent Cursor and Sensitive Fields

Sensitive fields may show:

- target geometry;
- field category;
- action state;
- safe label.

They must never show:

- password characters;
- OTP/TOTP values;
- credential values;
- other secret contents.

Redaction must happen before sensitive visual data reaches the normal frontend rendering pipeline.

CSS masking is not a security boundary.

---

## 15. Human Takeover

The browser must always make it clear when control belongs to the user versus the agent.

Provide an obvious takeover/pause mechanism when an agent task is active.

The user should be able to:

- pause the agent;
- resume the agent;
- take manual control;
- inspect the current page normally;
- return control to the agent.

Human takeover must create a real state transition in the agent runtime.

It must not be cosmetic UI.

---

## 16. Confirmation UX

Consequential actions should be confirmed in a browser-native way rather than through a generic modal spam pattern.

The confirmation surface should clearly state:

```text
Action
Target
Relevant parameters
Why confirmation is required
Confirm / Cancel
```

When biometric authorization is required, the native platform mechanism should be invoked from the trusted layer.

The browser UI should never fake a native biometric dialog.

---

## 17. Page-Level Agent Overlay

Agent UI may temporarily overlay the browser viewport when necessary.

Examples:

- target highlight;
- agent cursor;
- confirmation affordance;
- task status indicator;
- paused state;
- blocked state.

Overlays must be non-destructive.

They must not modify website DOM or semantics as the primary mechanism.

They must not intercept website pointer events unless the runtime intentionally enters an explicit user-confirmation interaction mode.

---

## 18. Side Panel

A traditional permanent AI side panel should NOT be the default layout.

A side panel may exist as an optional contextual surface for:

- task details;
- browser task history;
- downloads;
- credentials/settings when explicitly opened;
- verification evidence;
- agent activity details.

It should be collapsible and secondary to the browser viewport.

Default state should favor maximum webpage space.

---

## 19. Task Details

When the user inspects the active task, show structured information instead of a chat transcript.

Example:

```text
Task
Find the best option and open it

Current site
example.com

Status
Working

Current action
Opening result 3

Control
Pause / Take over
```

Do not expose internal model reasoning.

---

## 20. Browser Notifications / Prompts

Use compact browser-style notifications for low-risk events.

Examples:

```text
Download completed
Tab opened
Task completed
Agent paused
Navigation blocked
```

Use stronger confirmation surfaces for consequential actions.

Do not use giant modal dialogs for routine browser events.

---

## 21. Downloads

Downloads should feel like part of the browser rather than a separate file-management product.

Provide:

- visible download state;
- progress where available;
- completed state;
- failure state;
- quarantine/security status where relevant;
- reveal/open action through the appropriate platform path.

Do not expose unsafe files directly to web content.

---

## 22. Browser History / Bookmarks

When implemented, history and bookmarks should use familiar browser patterns.

Do not turn them into AI memory features.

Agent task history is a separate concept from browser history.

Keep them visually and semantically distinct.

---

## 23. Settings

Settings should look like a desktop application's browser settings, not an AI SaaS dashboard.

Recommended groups:

```text
General
Browser
Privacy & Security
Agent
Credentials
Appearance
Keyboard Shortcuts
Advanced
About
```

Only expose controls that correspond to real runtime behavior.

Do not expose fake configuration options.

---

## 24. Credential Vault UI

The Vault should be accessible from browser settings or a dedicated protected surface.

It should not dominate everyday browsing.

Vault UI should clearly distinguish:

```text
Browser credentials
Secure notes
TOTP
Identities
API keys
```

Do not expose secrets in normal browser views.

User-initiated reveal/copy actions must be deliberate and native/trusted where required.

---

## 25. Empty / New Tab

The new-tab page should remain useful as a browser start surface.

It should not become a giant marketing page or a generic chatbot screen.

A compact composition may include:

```text
Search / enter URL
Recent tabs/sites
Pinned shortcuts
Optional agent task entry
```

Avoid fake example cards that imply the agent has performed actions when it has not.

---

## 26. Error States

Error states should be browser-like, explicit, and actionable.

Examples:

```text
Page failed to load
Navigation blocked
Agent action failed
Agent paused
Verification inconclusive
Download failed
Browser crashed
```

Each state should expose an appropriate next action.

Do not show raw Rust errors to normal users.

Developer diagnostics may expose deeper details in a controlled debug surface.

---

## 27. Agent State Visual Language

Use a consistent state system.

Suggested states:

```text
IDLE
WORKING
WAITING
CONFIRMATION_REQUIRED
PAUSED
VERIFYING
COMPLETED
BLOCKED
FAILED
```

Each state should have:

- a consistent visual indicator;
- a clear textual label where needed;
- deterministic transitions.

Do not rely on color alone.

---

## 28. Motion Design

Use motion to explain behavior.

GSAP and Anime.js may be used extensively where they improve understanding of:

- agent cursor movement;
- target acquisition;
- action transitions;
- task progress;
- tab state transitions;
- panel opening/closing;
- confirmation transitions;
- verification results.

Do not animate every element.

Do not use animation merely to make a static UI look “AI-like.”

Motion should answer:

> What changed?

> What is the agent doing?

> What should the user notice?

Respect reduced-motion preferences.

---

## 29. Responsive Desktop Layout

This is a desktop application, not a responsive website.

Optimize for:

- macOS laptop displays;
- Windows laptops/desktops;
- high-DPI screens;
- resizable windows;
- narrow laptop widths;
- large external displays.

The browser viewport should receive the largest possible area.

Avoid layouts that become unusable when the window is narrowed.

---

## 30. Keyboard UX

Keyboard operation is essential.

Support browser-like shortcuts where feasible:

```text
Cmd/Ctrl + L     Focus address bar
Cmd/Ctrl + T     New tab
Cmd/Ctrl + W     Close tab
Cmd/Ctrl + Shift + T   Reopen tab
Cmd/Ctrl + R     Reload
Cmd/Ctrl + Tab   Next tab
Cmd/Ctrl + Shift + Tab Previous tab
Esc              Stop/cancel relevant interaction
```

Agent-specific shortcuts should be additive and documented.

Do not steal browser shortcuts unnecessarily.

---

## 31. Accessibility

The application shell must expose meaningful semantic roles and accessible labels.

Do not rely only on:

- color;
- animation;
- icon shape.

Respect:

- keyboard navigation;
- focus visibility;
- reduced motion;
- text scaling;
- contrast.

The browser shell and agent UI should remain usable when animation is reduced or disabled.

---

## 32. Window / Native Integration

macOS and Windows should feel native where appropriate.

Preserve platform conventions for:

- window controls;
- application menus;
- keyboard modifiers;
- notifications;
- secure prompts;
- file dialogs;
- biometric prompts;
- packaging.

Do not flatten platform differences merely to make the UI identical.

The product identity should come from the browser shell and typography, not from ignoring OS conventions.

---

## 33. Visual Hierarchy

The priority order is:

```text
1. Webpage content
2. Browser navigation
3. Agent activity when active
4. User confirmation when required
5. Secondary browser/application controls
6. Diagnostics/settings
```

The agent must not permanently cover the website.

---

## 34. Color Strategy

Do not use a default “AI purple/blue gradient” palette.

Use a restrained base palette with strong contrast and one or two purposeful accents.

Color should communicate state rather than decoration.

Examples:

```text
neutral → normal browser state
accent → active agent state
warning → requires attention
error → failed/blocked
success → verified completion
```

Do not encode state by color alone.

---

## 35. Typography

Typography should be intentionally selected rather than relying on generic browser-default UI typography everywhere.

Use:

- strong display/text hierarchy;
- compact browser chrome text;
- highly legible body text;
- clear labels for agent state;
- numerals that remain readable at small UI sizes.

Avoid excessive font weights and decorative typography inside browser chrome.

---

## 36. Iconography

Icons should be functional.

Prefer a small, coherent icon set.

Do not fill the interface with decorative icon buttons.

Every icon button must have:

- obvious purpose;
- tooltip/title where appropriate;
- accessible label.

Do not introduce icons merely to make empty space look designed.

---

## 37. Browser Identity vs Agent Identity

The browser should remain visually primary.

The agent identity should be visible through:

- Agent Cursor;
- agent activity indicator;
- task control;
- task state;
- concise agent affordances.

Do not brand every browser surface with the word “AI”.

The user should feel that the browser itself has autonomous capability.

---

## 38. Information Density

The browser must handle long-running, real work.

Avoid enormous cards and excessive whitespace inside the core browser UI.

Prefer:

- compact controls;
- clear grouping;
- strong spacing rhythm;
- predictable alignment.

The webpage should remain the densest and most information-rich surface.

---

## 39. Implementation Constraints

The UI must remain wired to real runtime state.

Do not build visual simulations that do not correspond to the Rust/browser runtime.

Specifically:

- Agent Cursor events must originate from real execution state.
- Agent status must originate from real TaskProgress/runtime state.
- Confirmation state must originate from Policy.
- Verification status must originate from Verification.
- Browser tabs must reflect real browser contexts.
- Downloads must reflect real download state.
- Errors must represent actual runtime failures.

No fake browser activity.

No fake agent activity.

---

## 40. Component Architecture

Organize frontend components around browser concepts rather than AI dashboard concepts.

Suggested structure:

```text
BrowserShell
├── WindowFrame
├── TabStrip
├── NavigationToolbar
│   ├── BackButton
│   ├── ForwardButton
│   ├── ReloadButton
│   ├── Omnibox
│   ├── BrowserActions
│   └── AgentControl
├── BrowserViewport
│   ├── BrowserSurface
│   ├── AgentOverlay
│   ├── AgentCursor
│   └── ContextualPrompt
├── OptionalSidePanel
└── StatusBar
```

Use the actual repository architecture if equivalent components already exist.

Do not create duplicate browser-shell implementations.

---

## 41. State Ownership

Browser state belongs to the browser runtime.

Agent state belongs to the agent runtime.

UI components render state; they do not invent it.

Do not maintain competing copies of:

- current tab;
- current browser URL;
- task state;
- agent state;
- target state;
- verification state.

---

## 42. No AI Dashboard Patterns

Explicitly avoid:

```text
chat-first homepage
persistent left AI sidebar
prompt history as main navigation
floating “Ask AI” button everywhere
large AI cards
fake agent analytics
AI-generated insight dashboards
```

The central object is always the browser page.

---

## 43. No Marketing UI Inside the Product

Do not use:

- hero sections;
- testimonials;
- pricing cards;
- feature marketing cards;
- fake example tasks;
- decorative illustrations;

inside the core application shell.

The user is already inside the product.

---

## 44. Impeccable Design Skill

Use the locally installed Impeccable skill for frontend design decisions and implementation quality where it is available in the development environment.

The skill is developer-local and must remain excluded from Git according to repository rules.

Do not copy the skill into product code.

Use it to improve the browser UI rather than creating a generic AI dashboard.

---

## 45. Definition of Done

The redesign is complete only when:

- the app visibly reads as a browser;
- tabs are primary task containers;
- navigation and omnibox feel browser-native;
- webpage content dominates the viewport;
- agent control is integrated into the browser shell;
- Agent Cursor is visibly connected to real agent actions;
- the physical mouse does not need to move;
- human takeover is obvious and functional;
- consequential confirmations are clear;
- sensitive fields remain protected;
- no fake browser/agent states exist;
- the UI works on macOS and Windows;
- the browser remains usable without the agent active;
- agent features enhance browsing instead of replacing the browser mental model.

---

## 46. Design Mantra

The product should communicate one idea immediately:

> **This is a browser that can act for you.**

Not:

> This is an AI dashboard with a browser inside it.

Every major UI decision should reinforce that distinction.
