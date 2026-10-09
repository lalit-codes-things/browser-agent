// Perception subsystem.
//
// C-58: track target/session/frame/loader identity and maintain a bounded
//        semantic state graph.
// C-80: per-frame perception with OOPIF aggregation; open shadow DOM
//        supported, closed shadow DOM not promised.
// C-81: accessibility/rendered-state/geometry analysis -> semantic
//        actionability filtering + mutation-noise suppression.
// C-82: NFKC + zero-width, bidi, homoglyph, mixed-script detection.
// C-83: semantic compression bounded by graph/context/attribute caps.
// C-84: fallback ladder.
// C-85: semantic references as stable abstractions.
// C-86: accessible-name vs rendered-text disagreement -> UNKNOWN.
// C-87: recursive OOPIF hit-testing (Phase 1 basic hit-test, Phase 3
//        complete recursive descent).
// C-88: hybrid typing (model cannot control JavaScript).
// C-89: canvas/virtualized-list limitations -> defined UNKNOWN semantics.
// C-59: semantic stabilization with livelock detector.
// C-141: actions bound to epoch from which they were derived; stale
//         actions rejected when relevant state changes before execution.
// C-65: Phase-1 controlled mutations.

pub mod actionability;
pub mod ax_extract;
pub mod compress;
pub mod deception;
pub mod dom_extract;
pub mod epoch;
pub mod fallback;
pub mod frames;
pub mod geometry;
pub mod graph;
pub mod livelock;
pub mod mutation_watcher;
pub mod noise;
pub mod pipeline;
pub mod reference;
pub mod shadow_dom;
pub mod stabilize;
pub mod unicode;
