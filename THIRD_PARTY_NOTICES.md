# Third-Party Notices

This file is an inventory scaffold, not a completed legal notice. Before distributing source bundles
or binaries, populate it from the exact locked and shipped dependency set and include the required
license texts/notices.

## Required inventory

Record each direct and transitive component that is distributed or linked, including at minimum:

- Rust crates and native libraries;
- npm packages bundled into the UI;
- Chromium Embedded Framework (CEF) and Chromium;
- llama.cpp and any selected inference backend;
- bundled fonts and other assets;
- installer/runtime components and update infrastructure.

For each component, record:

| Field             | Required value                                                      |
| ----------------- | ------------------------------------------------------------------- |
| Name              | Exact component/package name                                        |
| Version or commit | Pinned version/revision                                             |
| Source            | Canonical upstream source                                           |
| License           | SPDX identifier where valid, plus exact license text where required |
| Distribution      | Source-only, linked, bundled, or optional download                  |
| Notices           | Required attribution, notice, patent, or source-offer text          |
| Verification      | Source, hash, and review record                                     |

Do not assume CEF's license file covers every Chromium or bundled third-party component. Do not
accept unknown license metadata by default. Do not automatically accept GPL, AGPL, SSPL, or other
copyleft/commercially constrained terms without a documented compatibility review.

## Current status

**Incomplete — release blocker.** Generate the inventory from actual lockfiles and packaged
artifacts. Do not represent this scaffold as a complete third-party notice.
