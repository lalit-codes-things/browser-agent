//! Browser-Agent perception layer (BA-OBSERVATION-001, Sections 12.4-12.6).
//!
//! Perception builds a bounded semantic state graph scoped to one observation
//! epoch. Every node identity is valid only inside its epoch, and every target
//! reference is re-resolved here immediately before dispatch. All page-derived
//! text is untrusted observation data: it may inform the task but never
//! authorization.
//!
//! This crate defines the data contract and the invalidation rules. The actual
//! DOM/accessibility collection happens in the renderer-side observer that
//! implements the CEF message contract; this crate validates and stores what it
//! reports.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![deny(missing_docs)]

use std::collections::BTreeMap;

use ba_domain::{FrameId, NodeId, ObservationId, Origin, TabId, TargetRef};
use serde::{Deserialize, Serialize};

/// Maximum nodes retained from one observation.
pub const MAX_NODES: usize = 2_000;
/// Maximum actionable candidates retained from one observation.
pub const MAX_ACTIONABLE: usize = 256;
/// Maximum relationship depth serialized to the model.
pub const MAX_RELATION_DEPTH: u8 = 12;
/// Maximum serialized model-input tokens per observation.
pub const MAX_MODEL_INPUT_TOKENS: usize = 12_000;
/// Collection budget for one observation, excluding explicit navigation waits.
pub const COLLECTION_BUDGET_MS: u32 = 750;

/// Why an observation was invalidated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvalidationTrigger {
    /// A top-level navigation or redirect completed.
    TopLevelNavigation,
    /// The origin, scheme or port changed.
    OriginChanged,
    /// The active tab or window changed.
    ActiveTabChanged,
    /// A frame or renderer context was created, destroyed or replaced.
    RendererContextChanged,
    /// The browser crashed or reconnected.
    BrowserReconnected,
    /// The user typed, clicked or took over.
    UserInput,
    /// Relevant DOM or accessibility state mutated.
    DomMutation,
    /// Scroll, viewport, zoom or device scale changed.
    ViewportChanged,
    /// A popup or new target was created.
    PopupCreated,
    /// A permission or certificate interstitial appeared.
    Interstitial,
    /// A download transition changed page behaviour.
    DownloadTransition,
    /// A hit test did not match the intended target.
    HitTestMismatch,
}

/// A reason a stored target reference can no longer be used.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PerceptionError {
    /// No observation has been collected yet.
    #[error("no observation is available")]
    NoObservation,
    /// The reference belongs to a superseded epoch.
    #[error("the target belongs to a stale observation; re-observe before acting")]
    StaleEpoch,
    /// The node identity is not in the current observation.
    #[error("the target element is no longer present in the page")]
    UnknownNode,
    /// The frame identity changed or was replaced.
    #[error("the frame that owned the target was replaced")]
    FrameReplaced,
    /// The origin of the frame changed.
    #[error("the target's origin changed")]
    OriginChanged,
    /// The node is not actionable.
    #[error("the target element cannot receive that interaction")]
    NotActionable,
    /// The node is disabled.
    #[error("the target element is disabled")]
    Disabled,
    /// The node is not visible or has zero size.
    #[error("the target element is hidden or has no size")]
    NotVisible,
}

/// Geometry of one grounded element, in CSS pixels of its frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Geometry {
    /// Left edge.
    pub x: i32,
    /// Top edge.
    pub y: i32,
    /// Width.
    pub width: i32,
    /// Height.
    pub height: i32,
}

impl Geometry {
    /// Returns whether the box has area and is therefore hittable.
    #[must_use]
    pub const fn is_hittable(self) -> bool {
        self.width > 0 && self.height > 0
    }
}

/// One node of the semantic state graph.
///
/// `name` and `text` are page-controlled and untrusted, and have already been
/// redacted by the observer before construction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    /// Node identity, scoped to the observation epoch.
    pub node: NodeId,
    /// Frame that owns the node.
    pub frame: FrameId,
    /// Accessible role.
    pub role: String,
    /// Redacted accessible name.
    pub name: String,
    /// Redacted visible text, when relevant.
    pub text: String,
    /// Whether the node accepts interaction in this epoch.
    pub actionable: bool,
    /// Whether the node is visible to the user.
    pub visible: bool,
    /// Whether the node is enabled.
    pub enabled: bool,
    /// Bounding box.
    pub geometry: Geometry,
    /// Depth of the node inside the retained relationship graph.
    pub relation_depth: u8,
    /// Index of the parent node, when retained.
    pub parent: Option<usize>,
    /// Indices of retained child nodes.
    pub children: Vec<usize>,
}

/// Counts of data omitted by a bound, by role or class.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TruncationSummary {
    /// Nodes omitted because the node bound was reached.
    pub nodes_omitted_by_bound: usize,
    /// Actionable candidates omitted because the actionable bound was reached.
    pub actionable_omitted_by_bound: usize,
    /// Whether relationship depth was clamped.
    pub depth_clamped: bool,
    /// Whether the token budget was reached.
    pub token_budget_reached: bool,
    /// Omitted counts keyed by role.
    pub omitted_by_role: BTreeMap<String, usize>,
}

impl TruncationSummary {
    /// Returns whether any bound was reached.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes_omitted_by_bound == 0
            && self.actionable_omitted_by_bound == 0
            && !self.depth_clamped
            && !self.token_budget_reached
            && self.omitted_by_role.is_empty()
    }

    /// Returns the plain-language sentence describing what was left out.
    #[must_use]
    pub fn describe(&self) -> String {
        if self.is_empty() {
            return "The whole page view was observed within the collection limits.".to_owned();
        }
        let mut parts = vec![format!(
            "{} elements were beyond the observation limit",
            self.nodes_omitted_by_bound
        )];
        if self.actionable_omitted_by_bound > 0 {
            parts.push(format!(
                "{} additional interactive elements were not listed",
                self.actionable_omitted_by_bound
            ));
        }
        if self.depth_clamped {
            parts.push("deeper page structure was summarized".to_owned());
        }
        if self.token_budget_reached {
            parts.push("the model input budget was reached".to_owned());
        }
        format!("{}.", parts.join("; "))
    }
}

/// One bounded observation of a page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    /// Epoch identity. Every node identity is valid only inside this epoch.
    pub epoch: ObservationId,
    /// Tab that was observed.
    pub tab: TabId,
    /// Main frame of the observation.
    pub frame: FrameId,
    /// Exact origin observed.
    pub origin: Origin,
    /// Page title, redacted.
    pub title: String,
    /// Monotonic timestamp of collection, for latency accounting only.
    pub collected_at_monotonic_ms: u64,
    /// Measured collection duration.
    pub collection_ms: u32,
    /// Retained nodes.
    pub nodes: Vec<Node>,
    /// What was omitted by the observation bounds.
    pub truncation: TruncationSummary,
}

impl Observation {
    /// Returns whether the observation is still current for its context.
    #[must_use]
    pub fn matches_context(&self, tab: &TabId, frame: &FrameId, origin: &Origin) -> bool {
        &self.tab == tab && &self.frame == frame && self.origin.same_origin_exact(origin)
    }

    /// Returns the actionable candidates, capped at the schema bound.
    #[must_use]
    pub fn actionable(&self) -> Vec<&Node> {
        self.nodes
            .iter()
            .filter(|node| node.actionable && node.visible && node.enabled)
            .take(MAX_ACTIONABLE)
            .collect()
    }

    /// Returns the node for `id`, when retained.
    #[must_use]
    pub fn node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.iter().find(|node| &node.node == id)
    }
}

/// Requested properties of a target, verified against the current observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroundingRequest<'a> {
    /// Reference to resolve.
    pub target: &'a TargetRef,
    /// Tab that must still be active.
    pub tab: &'a TabId,
    /// Frame that must still own the target.
    pub frame: &'a FrameId,
    /// Origin that must still match exactly.
    pub origin: &'a Origin,
}

/// A resolved, still-valid target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroundedTarget {
    /// Node identity.
    pub node: NodeId,
    /// Frame identity.
    pub frame: FrameId,
    /// Observation epoch, unchanged by resolution.
    pub epoch: ObservationId,
    /// Accessible role, untrusted.
    pub role: String,
    /// Redacted accessible name, untrusted.
    pub name: String,
    /// Current bounding box.
    pub geometry: Geometry,
}

/// Holds the current observation and enforces epoch validity.
#[derive(Debug, Default)]
pub struct ObservationStore {
    current: Option<Observation>,
    invalidated_at_monotonic_ms: Option<u64>,
    last_trigger: Option<InvalidationTrigger>,
    epoch_changes: Vec<u64>,
}

impl ObservationStore {
    /// Creates an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the current observation.
    ///
    /// # Errors
    ///
    /// Returns [`PerceptionError::UnknownNode`] when a node references a parent
    /// or child index that does not exist, because a graph with dangling
    /// relationships cannot be used for grounding.
    pub fn store(&mut self, observation: Observation) -> Result<(), PerceptionError> {
        let len = observation.nodes.len();
        for node in &observation.nodes {
            if node.parent.is_some_and(|index| index >= len)
                || node.children.iter().any(|index| *index >= len)
            {
                return Err(PerceptionError::UnknownNode);
            }
        }
        self.current = Some(observation);
        self.invalidated_at_monotonic_ms = None;
        self.last_trigger = None;
        Ok(())
    }

    /// Returns the current observation, if one exists.
    #[must_use]
    pub const fn current(&self) -> Option<&Observation> {
        self.current.as_ref()
    }

    /// Invalidates the current observation.
    ///
    /// The observation is dropped rather than kept with a flag, so no code path
    /// can accidentally resolve a target from an invalidated epoch.
    pub fn invalidate(&mut self, trigger: InvalidationTrigger, monotonic_ms: u64) {
        self.current = None;
        self.invalidated_at_monotonic_ms = Some(monotonic_ms);
        self.last_trigger = Some(trigger);
        self.epoch_changes.push(monotonic_ms);
    }

    /// Returns whether a target must be re-observed before use.
    #[must_use]
    pub fn requires_fresh_observation(&self) -> bool {
        self.current.is_none()
    }

    /// Returns the trigger that invalidated the last observation.
    #[must_use]
    pub const fn last_trigger(&self) -> Option<InvalidationTrigger> {
        self.last_trigger
    }

    /// Counts epoch changes inside a rolling window.
    #[must_use]
    pub fn epoch_changes_within(&self, window_ms: u64, now_ms: u64) -> usize {
        let cutoff = now_ms.saturating_sub(window_ms);
        self.epoch_changes
            .iter()
            .filter(|change| **change >= cutoff)
            .count()
    }

    /// Resolves a target reference against the current observation.
    ///
    /// # Errors
    ///
    /// Returns [`PerceptionError`] for a missing, stale, replaced or
    /// non-actionable target. Every failure means the caller must re-observe
    /// instead of dispatching.
    pub fn resolve(
        &self,
        request: &GroundingRequest<'_>,
    ) -> Result<GroundedTarget, PerceptionError> {
        let observation = self
            .current
            .as_ref()
            .ok_or(PerceptionError::NoObservation)?;
        if observation.tab != *request.tab {
            return Err(PerceptionError::FrameReplaced);
        }
        if observation.epoch != request.target.epoch {
            return Err(PerceptionError::StaleEpoch);
        }
        if observation.frame != *request.frame || request.target.frame != *request.frame {
            return Err(PerceptionError::FrameReplaced);
        }
        if !observation.origin.same_origin_exact(request.origin)
            || !request.target.origin.same_origin_exact(request.origin)
        {
            return Err(PerceptionError::OriginChanged);
        }
        let node = observation
            .nodes
            .iter()
            .find(|node| node.node == request.target.node)
            .ok_or(PerceptionError::UnknownNode)?;
        if !node.visible || !node.geometry.is_hittable() {
            return Err(PerceptionError::NotVisible);
        }
        if !node.enabled {
            return Err(PerceptionError::Disabled);
        }
        if !node.actionable {
            return Err(PerceptionError::NotActionable);
        }
        Ok(GroundedTarget {
            node: node.node.clone(),
            frame: node.frame.clone(),
            epoch: observation.epoch.clone(),
            role: node.role.clone(),
            name: node.name.clone(),
            geometry: node.geometry,
        })
    }
}

/// Applies the schema bounds to a raw node list and reports what was omitted.
///
/// The observer calls this before publishing, so a page that inflates its DOM
/// cannot push the model input past its budget. Omitted data is reported
/// explicitly and never silently dropped when it changes action safety.
#[must_use]
pub fn apply_bounds(
    mut nodes: Vec<Node>,
    estimated_tokens_per_node: usize,
) -> (Vec<Node>, TruncationSummary) {
    let mut truncation = TruncationSummary::default();
    let token_budget_nodes = MAX_MODEL_INPUT_TOKENS
        .checked_div(estimated_tokens_per_node)
        .unwrap_or(MAX_NODES);
    let effective_bound = MAX_NODES.min(token_budget_nodes.max(1));
    if nodes.len() > effective_bound {
        for node in nodes.drain(effective_bound..) {
            *truncation.omitted_by_role.entry(node.role).or_insert(0) += 1;
            truncation.nodes_omitted_by_bound += 1;
        }
    }
    if token_budget_nodes < MAX_NODES {
        truncation.token_budget_reached = true;
    }
    let mut actionable_seen = 0usize;
    for node in &nodes {
        if node.actionable && node.visible && node.enabled {
            actionable_seen += 1;
            if actionable_seen > MAX_ACTIONABLE {
                truncation.actionable_omitted_by_bound += 1;
            }
        }
    }
    if nodes
        .iter()
        .any(|node| node.relation_depth > MAX_RELATION_DEPTH)
    {
        truncation.depth_clamped = true;
    }
    (nodes, truncation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ba_domain::NodeId;

    fn node(index: usize) -> Node {
        Node {
            node: NodeId::parse(format!("node-{index}")).unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
            role: "button".to_owned(),
            name: format!("Control {index}"),
            text: String::new(),
            actionable: true,
            visible: true,
            enabled: true,
            geometry: Geometry {
                x: 0,
                y: 0,
                width: 100,
                height: 32,
            },
            relation_depth: 3,
            parent: None,
            children: Vec::new(),
        }
    }

    fn observation(nodes: Vec<Node>) -> Observation {
        Observation {
            epoch: ObservationId::parse("epoch-1").unwrap(),
            tab: TabId::parse("tab-1").unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            title: "Example".to_owned(),
            collected_at_monotonic_ms: 1_000,
            collection_ms: 120,
            nodes,
            truncation: TruncationSummary::default(),
        }
    }

    fn target(node_id: &str, epoch: &str) -> TargetRef {
        TargetRef {
            node: NodeId::parse(node_id).unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
            tab: TabId::parse("tab-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse(epoch).unwrap(),
            role: None,
            label: None,
        }
    }

    #[test]
    fn resolves_a_live_target() {
        let mut store = ObservationStore::new();
        store.store(observation(vec![node(0), node(1)])).unwrap();
        let target = target("node-1", "epoch-1");
        let request = GroundingRequest {
            target: &target,
            tab: &TabId::parse("tab-1").unwrap(),
            frame: &FrameId::parse("frame-1").unwrap(),
            origin: &Origin::parse("https://example.com").unwrap(),
        };
        let grounded = store.resolve(&request).unwrap();
        assert_eq!(grounded.node.as_str(), "node-1");
        assert_eq!(grounded.geometry.width, 100);
    }

    #[test]
    fn rejects_stale_epochs_and_replaced_contexts() {
        let mut store = ObservationStore::new();
        store.store(observation(vec![node(0)])).unwrap();

        let stale = target("node-0", "epoch-0");
        assert_eq!(
            store
                .resolve(&GroundingRequest {
                    target: &stale,
                    tab: &TabId::parse("tab-1").unwrap(),
                    frame: &FrameId::parse("frame-1").unwrap(),
                    origin: &Origin::parse("https://example.com").unwrap(),
                })
                .unwrap_err(),
            PerceptionError::StaleEpoch
        );

        let live = target("node-0", "epoch-1");
        assert_eq!(
            store
                .resolve(&GroundingRequest {
                    target: &live,
                    tab: &TabId::parse("tab-2").unwrap(),
                    frame: &FrameId::parse("frame-1").unwrap(),
                    origin: &Origin::parse("https://example.com").unwrap(),
                })
                .unwrap_err(),
            PerceptionError::FrameReplaced
        );
        assert_eq!(
            store
                .resolve(&GroundingRequest {
                    target: &live,
                    tab: &TabId::parse("tab-1").unwrap(),
                    frame: &FrameId::parse("frame-1").unwrap(),
                    origin: &Origin::parse("https://elsewhere.test").unwrap(),
                })
                .unwrap_err(),
            PerceptionError::OriginChanged
        );
    }

    #[test]
    fn invalidation_drops_the_observation_entirely() {
        let mut store = ObservationStore::new();
        store.store(observation(vec![node(0)])).unwrap();
        store.invalidate(InvalidationTrigger::DomMutation, 2_000);
        assert!(store.current().is_none());
        assert!(store.requires_fresh_observation());
        assert_eq!(store.last_trigger(), Some(InvalidationTrigger::DomMutation));
        let live = target("node-0", "epoch-1");
        assert_eq!(
            store
                .resolve(&GroundingRequest {
                    target: &live,
                    tab: &TabId::parse("tab-1").unwrap(),
                    frame: &FrameId::parse("frame-1").unwrap(),
                    origin: &Origin::parse("https://example.com").unwrap(),
                })
                .unwrap_err(),
            PerceptionError::NoObservation
        );
    }

    #[test]
    fn hidden_disabled_and_non_actionable_targets_are_rejected() {
        let cases = [
            Node {
                visible: false,
                ..node(0)
            },
            Node {
                geometry: Geometry {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
                ..node(0)
            },
            Node {
                enabled: false,
                ..node(0)
            },
            Node {
                actionable: false,
                ..node(0)
            },
        ];
        let expected = [
            PerceptionError::NotVisible,
            PerceptionError::NotVisible,
            PerceptionError::Disabled,
            PerceptionError::NotActionable,
        ];
        for (node, expected) in cases.into_iter().zip(expected) {
            let mut store = ObservationStore::new();
            store.store(observation(vec![node])).unwrap();
            let live = target("node-0", "epoch-1");
            assert_eq!(
                store
                    .resolve(&GroundingRequest {
                        target: &live,
                        tab: &TabId::parse("tab-1").unwrap(),
                        frame: &FrameId::parse("frame-1").unwrap(),
                        origin: &Origin::parse("https://example.com").unwrap(),
                    })
                    .unwrap_err(),
                expected
            );
        }
    }

    #[test]
    fn dangling_graph_references_are_rejected() {
        let mut store = ObservationStore::new();
        let dangling = Node {
            parent: Some(9),
            ..node(0)
        };
        assert_eq!(
            store.store(observation(vec![dangling])).unwrap_err(),
            PerceptionError::UnknownNode
        );
        assert!(store.current().is_none());
    }

    #[test]
    fn bounds_report_what_was_omitted() {
        let nodes: Vec<Node> = (0..MAX_NODES + 5).map(node).collect();
        let (retained, truncation) = apply_bounds(nodes, 6);
        assert_eq!(retained.len(), MAX_NODES);
        assert_eq!(truncation.nodes_omitted_by_bound, 5);
        assert_eq!(truncation.omitted_by_role.get("button"), Some(&5));
        assert!(truncation.describe().contains("5 elements were beyond"));
    }

    #[test]
    fn token_budget_clamps_before_the_node_bound() {
        let nodes: Vec<Node> = (0..300).map(node).collect();
        let (retained, truncation) = apply_bounds(nodes, 64);
        assert!(retained.len() < 300);
        assert!(truncation.token_budget_reached);
        assert!(truncation.describe().contains("model input budget"));
    }

    #[test]
    fn actionable_candidates_are_capped() {
        let nodes: Vec<Node> = (0..MAX_NODES).map(node).collect();
        let (retained, truncation) = apply_bounds(nodes, 6);
        assert_eq!(retained.len(), MAX_NODES);
        assert_eq!(
            truncation.actionable_omitted_by_bound,
            MAX_NODES - MAX_ACTIONABLE
        );
    }

    #[test]
    fn epoch_change_rate_is_measurable() {
        let mut store = ObservationStore::new();
        for ms in [100, 200, 300, 5_000] {
            store.invalidate(InvalidationTrigger::DomMutation, ms);
        }
        // Rolling window: only the change at 5,000 ms is inside the last 2 s.
        assert_eq!(store.epoch_changes_within(2_000, 5_000), 1);
        // A 2 s window at 400 ms reaches back past zero, so all four count.
        assert_eq!(store.epoch_changes_within(2_000, 400), 4);
    }
}
