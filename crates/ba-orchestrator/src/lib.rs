//! Browser-Agent task orchestrator (Section 12.2, BA-NORMALIZE-001,
//! BA-WIN-002/003, BA-STUCK-001, BA-RECOVERY-001).
//!
//! The orchestrator owns the task state machine, the task budget, the approval
//! book and the no-progress and livelock protections. It emits every
//! user-visible state change as a validated [`ba_domain::TaskEvent`]; nothing
//! else may synthesize task state.
//!
//! Exactly one autonomous task may be active across the whole application
//! process. A second submission is refused with an explicit message rather than
//! queued (BA-WIN-003).

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![deny(missing_docs)]

mod budget;

pub use budget::{BudgetDimension, BudgetError, TaskBudget};

use ba_domain::{
    ActionClass, ActionDigest, ActionId, ActionKind, MAX_TASK_TEXT_CODEPOINTS, Origin, ProfileKind,
    ReadinessState, Revision, STUCK_CONTROLS, STUCK_LABEL, Sequence, TabId, TaskBudgetView,
    TaskEvent, TaskId, TaskState, TaskStateError, UnixMillis, VerificationOutcome, WaitReason,
    WindowId, copy_table_json, is_secret_like, mask_secret_like, readiness_controls,
    readiness_label, task_state_label,
};
use ba_executor::{DispatchBlocked, DispatchOutcome, PortOutcome};
use ba_perception::ObservationStore;
use ba_policy::{
    ApprovalBinding, ApprovalError, ApprovalGrant, ApprovalStore, ClearCause, HandoffPresentation,
    InvalidationReason, PendingApproval, PendingApprovalRequest, PolicyDecision, PolicyEngine,
    PolicyError, PolicyOutcome, PolicyRequest, RuleId,
};
use ba_verify::VerificationReport;

/// Maximum semantic epoch changes inside a rolling window before pausing.
pub const LIVELOCK_EPOCH_CHANGES: usize = 8;
/// Rolling window for the livelock check.
pub const LIVELOCK_WINDOW_MS: u64 = 2_000;
/// Clarification rounds allowed per task (BA-NORMALIZE-001).
pub const MAX_CLARIFICATION_ROUNDS: u8 = 2;

/// Context for submitting a task.
#[derive(Debug, Clone)]
pub struct SubmitContext {
    /// Window the task was submitted from.
    pub window: WindowId,
    /// Tab that owns the task.
    pub tab: TabId,
    /// Origin of that tab.
    pub origin: Origin,
    /// Which profile the tab belongs to.
    pub profile: ProfileKind,
}

/// Result of a submission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmitOutcome {
    /// The task was recorded and is now `CREATED`.
    Accepted {
        /// New task identity.
        task_id: TaskId,
        /// Events emitted by the submission.
        events: Vec<TaskEvent>,
    },
    /// The instruction looked like it contained a literal secret, so the user
    /// is asked to use the vault pathway instead (BA-TRANSPARENCY-001).
    NeedsVaultInstead {
        /// Plain-language explanation with the masked instruction.
        detail: String,
    },
}

/// Orchestrator failures. Every variant is fail-closed: none of them can be
/// read as permission to continue.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OrchestratorError {
    /// Another task is already active process-wide.
    #[error("another Agent task is already running")]
    TaskAlreadyRunning {
        /// Owner of the running task.
        owner_task_id: TaskId,
        /// Window that owns it.
        owner_window: WindowId,
    },
    /// The instruction was empty.
    #[error("the task instruction is empty")]
    EmptyTask,
    /// The instruction exceeded the composer limit.
    #[error("the instruction is {actual} code points, over the {limit} limit")]
    TaskTooLong {
        /// Measured length.
        actual: usize,
        /// Limit.
        limit: usize,
    },
    /// The Agent is disabled in private windows.
    #[error("the Agent is unavailable in Private Windows")]
    PrivateWindowAgentDisabled,
    /// No task is active.
    #[error("no Agent task is active")]
    NoActiveTask,
    /// The request belongs to a different task.
    #[error("event or request belongs to a different task")]
    TaskMismatch,
    /// The transition is not allowed.
    #[error(transparent)]
    Transition(#[from] TaskStateError),
    /// A decision named an approval that is not pending.
    #[error(transparent)]
    Approval(#[from] ApprovalError),
    /// A budget extension could not be applied.
    #[error(transparent)]
    Budget(#[from] BudgetError),
    /// Policy could not evaluate the request.
    #[error(transparent)]
    Policy(#[from] PolicyError),
    /// Two clarification rounds were already used.
    #[error(
        "two clarification rounds were already used; state assumptions and continue only safely"
    )]
    ClarificationLimitReached,
}

/// A proposal's disposition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalDisposition {
    /// Policy authorized the action once.
    Authorized {
        /// The single-use authorization the executor must consume.
        authorization: Box<ba_policy::Authorization>,
    },
    /// One typed user decision is required.
    AwaitingApproval {
        /// The pending request.
        approval: Box<PendingApproval>,
        /// Whether the agent may execute after approval.
        approvable: bool,
    },
    /// The user must perform the step themselves.
    HandedToUser {
        /// Presentation for the handoff prompt.
        presentation: Box<HandoffPresentation>,
    },
    /// Policy refused the action.
    Refused {
        /// Rule that refused it.
        rule: RuleId,
        /// Plain-language reason.
        reason: String,
    },
    /// The task stopped because a budget ceiling was reached.
    BudgetReached,
}

/// The result of proposing one action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalOutcome {
    /// What happened to the proposal.
    pub disposition: ProposalDisposition,
    /// Trusted consequence class.
    pub class: ActionClass,
    /// Events emitted.
    pub events: Vec<TaskEvent>,
}

/// A finished task, kept for the local task history view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRecord {
    /// Task identity.
    pub task_id: TaskId,
    /// The original instruction, masked for secret-shaped literals.
    pub original_text: String,
    /// Terminal lifecycle state.
    pub state: TaskState,
    /// Independent verification outcome, when one applies.
    pub verification: Option<VerificationOutcome>,
    /// When the task was created.
    pub created_at_unix_ms: UnixMillis,
    /// When it reached a terminal state.
    pub ended_at_unix_ms: UnixMillis,
    /// Every event the task emitted.
    pub events: Vec<TaskEvent>,
    /// How many actions were dispatched.
    pub dispatched_actions: u64,
    /// Whether the task used its budget extension.
    pub extension_used: bool,
}

impl TaskRecord {
    /// Returns the summary sentence for the history list.
    #[must_use]
    pub fn summary_line(&self) -> String {
        let outcome = self
            .verification
            .map_or("Verification not run", ba_domain::verification_label);
        format!("{} \u{2014} {outcome}", self.state)
    }

    /// Returns the prefill text for `Run a similar task again`.
    ///
    /// The text is masked and is never submitted automatically.
    #[must_use]
    pub fn prefill_text(&self) -> String {
        mask_secret_like(&self.original_text).into_owned()
    }
}

#[derive(Debug)]
struct ActiveTask {
    id: TaskId,
    revision: Revision,
    state: TaskState,
    text: String,
    wait_reason: Option<WaitReason>,
    budget: TaskBudget,
    window: WindowId,
    tab: TabId,
    origin: Origin,
    owned_tabs: Vec<TabId>,
    events: Vec<TaskEvent>,
    last_digest: Option<ActionDigest>,
    no_progress_repeats: u8,
    stuck: bool,
    clarification_rounds: u8,
    waiting_since_ms: Option<u64>,
    created_at_unix_ms: UnixMillis,
    verification: Option<VerificationOutcome>,
    dispatched_actions: u64,
    in_flight: bool,
    stopped_by_user: bool,
}

/// The task orchestrator.
pub struct Orchestrator {
    policy: PolicyEngine,
    approvals: ApprovalStore,
    observations: ObservationStore,
    sequence: Sequence,
    monotonic_ms: u64,
    active: Option<ActiveTask>,
    history: Vec<TaskRecord>,
    last_finished: Option<TaskRecord>,
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Orchestrator {
    /// Creates an orchestrator with no active task.
    #[must_use]
    pub fn new() -> Self {
        Self {
            policy: PolicyEngine::new(),
            approvals: ApprovalStore::new(),
            observations: ObservationStore::new(),
            sequence: Sequence::new(1),
            monotonic_ms: 0,
            active: None,
            history: Vec::new(),
            last_finished: None,
        }
    }

    /// Returns the canonical copy table JSON so the runtime and the UI share one
    /// source (BA-STATE-001).
    #[must_use]
    pub fn copy_contract() -> String {
        copy_table_json()
    }

    /// Returns the model readiness copy and its supported controls.
    #[must_use]
    pub fn readiness_copy(state: ReadinessState) -> (&'static str, &'static [&'static str]) {
        (readiness_label(state), readiness_controls(state))
    }

    /// Returns the stuck-state controls (BA-STUCK-001).
    #[must_use]
    pub const fn stuck_controls() -> [&'static str; 3] {
        STUCK_CONTROLS
    }

    /// Returns the active task identity.
    #[must_use]
    pub fn active_task_id(&self) -> Option<&TaskId> {
        self.active.as_ref().map(|task| &task.id)
    }

    /// Returns the active task state.
    #[must_use]
    pub fn active_state(&self) -> Option<TaskState> {
        self.active.as_ref().map(|task| task.state)
    }

    /// Returns the window that owns the active task.
    #[must_use]
    pub fn owner_window(&self) -> Option<&WindowId> {
        self.active.as_ref().map(|task| &task.window)
    }

    /// Returns the tab the active task is working in.
    #[must_use]
    pub fn active_tab(&self) -> Option<&TabId> {
        self.active.as_ref().map(|task| &task.tab)
    }

    /// Returns the exact origin the active task is bound to.
    #[must_use]
    pub fn active_origin(&self) -> Option<&Origin> {
        self.active.as_ref().map(|task| &task.origin)
    }

    /// Returns the tabs in the task scope.
    #[must_use]
    pub fn task_tabs(&self) -> Vec<TabId> {
        self.active
            .as_ref()
            .map(|task| task.owned_tabs.clone())
            .unwrap_or_default()
    }

    /// Returns the pending approval, if one exists.
    #[must_use]
    pub fn pending_approval(&self) -> Option<&PendingApproval> {
        self.approvals.pending()
    }

    /// Returns the active task's budget view.
    #[must_use]
    pub fn budget(&self) -> Option<TaskBudgetView> {
        self.active.as_ref().map(|task| task.budget.view())
    }

    /// Returns every event of the active task.
    #[must_use]
    pub fn active_events(&self) -> Vec<TaskEvent> {
        self.active
            .as_ref()
            .map(|task| task.events.clone())
            .unwrap_or_default()
    }

    /// Returns the completed task history, oldest first.
    #[must_use]
    pub fn history(&self) -> &[TaskRecord] {
        &self.history
    }

    /// Returns the most recently finished task so the panel can still show its
    /// result after the task left the active slot.
    #[must_use]
    pub const fn last_finished(&self) -> Option<&TaskRecord> {
        self.last_finished.as_ref()
    }

    /// Returns whether the agent may dispatch a new action right now.
    #[must_use]
    pub fn may_dispatch(&self) -> bool {
        match &self.active {
            Some(task) => {
                matches!(
                    task.state,
                    TaskState::Executing
                        | TaskState::Observing
                        | TaskState::Proposing
                        | TaskState::PolicyCheck
                ) && !task.in_flight
                    && !task.stopped_by_user
                    && !task.stuck
            }
            None => false,
        }
    }

    /// Submits a new task instruction.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when a task is already active, the
    /// instruction is empty or too long, or the submitting tab is private.
    pub fn submit(
        &mut self,
        text: &str,
        context: SubmitContext,
        now: UnixMillis,
    ) -> Result<SubmitOutcome, OrchestratorError> {
        if let Some(task) = &self.active {
            return Err(OrchestratorError::TaskAlreadyRunning {
                owner_task_id: task.id.clone(),
                owner_window: task.window.clone(),
            });
        }
        if context.profile == ProfileKind::Private {
            return Err(OrchestratorError::PrivateWindowAgentDisabled);
        }
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(OrchestratorError::EmptyTask);
        }
        let length = trimmed.chars().count();
        if length > MAX_TASK_TEXT_CODEPOINTS {
            return Err(OrchestratorError::TaskTooLong {
                actual: length,
                limit: MAX_TASK_TEXT_CODEPOINTS,
            });
        }
        if is_secret_like(trimmed) {
            return Ok(SubmitOutcome::NeedsVaultInstead {
                detail: format!(
                    "This instruction appears to contain a literal secret, so it was not sent to the model. Use the saved-login pathway instead. Instruction as recorded: {}",
                    mask_secret_like(trimmed)
                ),
            });
        }

        let mut sequence = self.sequence;
        let task_id = next_task_id(&mut sequence)?;
        self.sequence = sequence;
        let monotonic_ms = self.monotonic_ms;
        let mut task = ActiveTask {
            id: task_id.clone(),
            revision: Revision::new(1),
            state: TaskState::Created,
            text: trimmed.to_owned(),
            wait_reason: None,
            budget: TaskBudget::reference_profile(),
            window: context.window,
            tab: context.tab.clone(),
            origin: context.origin,
            owned_tabs: vec![context.tab],
            events: Vec::new(),
            last_digest: None,
            no_progress_repeats: 0,
            stuck: false,
            clarification_rounds: 0,
            waiting_since_ms: None,
            created_at_unix_ms: now,
            verification: None,
            dispatched_actions: 0,
            in_flight: false,
            stopped_by_user: false,
        };
        let event = build_event(
            &mut self.sequence,
            &task,
            EventSpec {
                state: TaskState::Created,
                wait_reason: None,
                detail: "Instruction recorded. No page action has been taken.",
                action_id: None,
                class: None,
            },
            now,
            monotonic_ms,
        );
        task.events.push(event);
        let events = task.events.clone();
        self.active = Some(task);
        Ok(SubmitOutcome::Accepted { task_id, events })
    }

    /// Moves a fresh task into observation.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when no task is active or the transition is
    /// illegal.
    pub fn begin_observation(
        &mut self,
        now: UnixMillis,
    ) -> Result<Vec<TaskEvent>, OrchestratorError> {
        self.require_active()?;
        let mut events =
            self.transition(TaskState::Normalizing, None, "Preparing the task.", now)?;
        events.extend(self.transition(
            TaskState::Observing,
            None,
            "Inspecting the current page.",
            now,
        )?);
        Ok(events)
    }

    /// Records one clarification round (BA-NORMALIZE-001).
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError::ClarificationLimitReached`] after two rounds
    /// and [`OrchestratorError::NoActiveTask`] when nothing is active.
    pub fn ask_clarification(
        &mut self,
        question: &str,
        now: UnixMillis,
    ) -> Result<Vec<TaskEvent>, OrchestratorError> {
        let rounds = self.require_active()?.clarification_rounds;
        if rounds >= MAX_CLARIFICATION_ROUNDS {
            return Err(OrchestratorError::ClarificationLimitReached);
        }
        if let Some(task) = self.active.as_mut() {
            task.clarification_rounds += 1;
        }
        let detail = format!("Needs your input: {}", mask_secret_like(question));
        self.transition(
            TaskState::WaitingForUser,
            Some(WaitReason::UserInput),
            &detail,
            now,
        )
    }

    /// Evaluates one proposed action through policy and records the outcome.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when no task is active, the request belongs
    /// to another task, the transition is illegal, or policy cannot evaluate it.
    pub fn propose(
        &mut self,
        request: &PolicyRequest<'_>,
        now: UnixMillis,
    ) -> Result<ProposalOutcome, OrchestratorError> {
        let task_id = self
            .active
            .as_ref()
            .ok_or(OrchestratorError::NoActiveTask)?
            .id
            .clone();
        if request.context.task_id != task_id {
            return Err(OrchestratorError::TaskMismatch);
        }

        let mut events =
            self.transition(TaskState::Proposing, None, "Preparing the next step.", now)?;
        if !self.consume(BudgetDimension::ActionProposals, 1) {
            events.extend(self.reach_budget_ceiling(now)?);
            return Ok(ProposalOutcome {
                disposition: ProposalDisposition::BudgetReached,
                class: ActionClass::Unknown,
                events,
            });
        }
        events.extend(self.transition(
            TaskState::PolicyCheck,
            None,
            "Checking permission for the proposed step.",
            now,
        )?);

        let outcome: PolicyOutcome = self.policy.evaluate(request, now)?;
        let class = outcome.class;
        let disposition = match outcome.decision {
            PolicyDecision::Allow(authorization) => {
                events.extend(self.transition(
                    TaskState::Executing,
                    None,
                    "Performing the approved step.",
                    now,
                )?);
                if let Some(task) = self.active.as_mut() {
                    task.in_flight = true;
                }
                ProposalDisposition::Authorized { authorization }
            }
            PolicyDecision::RequireApproval {
                rule,
                presentation,
                approvable,
            } => {
                let sequence = self.sequence;
                self.sequence = sequence.next().unwrap_or(sequence);
                let action_id = ActionId::from_number("action", sequence.get());
                let approval = self
                    .approvals
                    .request(
                        PendingApprovalRequest {
                            action_id: action_id.clone(),
                            task_id: task_id.clone(),
                            digest: request.action.digest(),
                            origin: request.context.origin.clone(),
                            frame: request.context.frame.clone(),
                            epoch: request.context.epoch.clone(),
                            rule,
                            presentation: (*presentation).clone(),
                        },
                        now,
                    )?
                    .clone();
                events.extend(self.transition_with_action(
                    TaskState::WaitingForUser,
                    Some(WaitReason::ActionApproval),
                    rule.plain_language(),
                    Some(action_id),
                    Some(class),
                    now,
                )?);
                ProposalDisposition::AwaitingApproval {
                    approval: Box::new(approval),
                    approvable,
                }
            }
            PolicyDecision::HandoffRequired { presentation } => {
                if let Some(task) = self.active.as_mut() {
                    task.stopped_by_user = true;
                }
                events.extend(self.transition(
                    TaskState::UserTakeover,
                    None,
                    "The agent stopped and you have control. Perform this step yourself, then resume.",
                    now,
                )?);
                ProposalDisposition::HandedToUser { presentation }
            }
            PolicyDecision::Refuse { rule, reason } => {
                events.extend(self.transition(TaskState::Blocked, None, &reason, now)?);
                ProposalDisposition::Refused { rule, reason }
            }
        };
        Ok(ProposalOutcome {
            disposition,
            class,
            events,
        })
    }

    /// Records the executor's result for the in-flight action.
    ///
    /// The transition chosen always comes from the normative table: a result
    /// that never reached the page moves to `VERIFYING` and then `OBSERVING`, a
    /// user-controlled browser moves to `USER_TAKEOVER`, and a stopped policy or
    /// audit layer fails the task.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when no task is active or the transition is
    /// illegal.
    pub fn record_dispatch(
        &mut self,
        outcome: &DispatchOutcome,
        kind: ActionKind,
        normalized_digest: ActionDigest,
        now: UnixMillis,
    ) -> Result<Vec<TaskEvent>, OrchestratorError> {
        self.require_active()?;
        if let Some(task) = self.active.as_mut() {
            task.in_flight = false;
        }
        let mut events = Vec::new();

        if let PortOutcome::NotDispatched { reason } = &outcome.port {
            return match reason {
                DispatchBlocked::UserInControl => {
                    if let Some(task) = self.active.as_mut() {
                        task.stopped_by_user = true;
                    }
                    self.transition(TaskState::UserTakeover, None, reason.plain_language(), now)
                }
                DispatchBlocked::PolicyStopped => {
                    self.transition(TaskState::Failed, None, reason.plain_language(), now)
                }
                DispatchBlocked::HostUnavailable
                | DispatchBlocked::StaleEpoch
                | DispatchBlocked::TargetMissing
                | DispatchBlocked::FrameReplaced => {
                    let mut events =
                        self.transition(TaskState::Verifying, None, reason.plain_language(), now)?;
                    events.extend(self.transition(
                        TaskState::Observing,
                        None,
                        "Waiting to inspect the page again before the next step.",
                        now,
                    )?);
                    Ok(events)
                }
            };
        }

        let navigation = kind == ActionKind::Navigate;
        let _ = self.consume(BudgetDimension::DispatchedActions, 1);
        if navigation {
            let _ = self.consume(BudgetDimension::Navigations, 1);
        }
        if let Some(task) = self.active.as_mut() {
            task.dispatched_actions = task.dispatched_actions.saturating_add(1);
            let repeats = match task.last_digest.as_ref() {
                Some(previous) if *previous == normalized_digest => task.no_progress_repeats + 1,
                _ => 1,
            };
            task.no_progress_repeats = repeats;
            task.last_digest = Some(normalized_digest);
        }
        let ambiguous = matches!(outcome.port, PortOutcome::Ambiguous { .. });
        if ambiguous && let Some(task) = self.active.as_mut() {
            // An ambiguous dispatch may already have reached the page, so the
            // outcome starts as UNKNOWN and is never reported as success.
            task.verification = Some(VerificationOutcome::Unknown);
        }
        events.extend(self.transition(
            TaskState::Verifying,
            None,
            "Checking the result of the step.",
            now,
        )?);

        let exhausted = self
            .active
            .as_ref()
            .map(|task| task.budget.exhausted())
            .unwrap_or_default();
        if !exhausted.is_empty() {
            events.extend(self.transition(
                TaskState::WaitingForUser,
                Some(WaitReason::TaskBudget),
                "A task budget ceiling was reached. Choose how to continue.",
                now,
            )?);
        }
        Ok(events)
    }

    /// Records that the page changed meaningfully, resetting no-progress state.
    pub fn note_semantic_change(&mut self) {
        if let Some(task) = self.active.as_mut() {
            task.no_progress_repeats = 0;
            task.stuck = false;
        }
    }

    /// Records the verifier's report for the just-executed step.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when no task is active or the transition is
    /// illegal.
    pub fn record_verification(
        &mut self,
        report: &VerificationReport,
        terminal: bool,
        now: UnixMillis,
    ) -> Result<Vec<TaskEvent>, OrchestratorError> {
        self.require_active()?;
        let stuck = self
            .active
            .as_ref()
            .is_some_and(|task| task.no_progress_repeats >= 2);
        if stuck {
            if let Some(task) = self.active.as_mut() {
                task.stuck = true;
                task.verification = Some(report.outcome);
            }
            // `VERIFYING` may only reach `OBSERVING` before stopping, so the
            // pause goes through a legal intermediate state rather than
            // jumping.
            let mut events = self.transition(
                TaskState::Observing,
                None,
                "No progress was observed, so the agent stopped before repeating the step.",
                now,
            )?;
            let detail = format!("{STUCK_LABEL}. {}", STUCK_CONTROLS.join(" / "));
            events.extend(self.transition(TaskState::Paused, None, &detail, now)?);
            return Ok(events);
        }
        if terminal {
            if let Some(task) = self.active.as_mut() {
                task.verification = Some(report.outcome);
            }
            let state = match report.outcome {
                VerificationOutcome::VerifiedFailure => TaskState::Failed,
                VerificationOutcome::VerifiedSuccess
                | VerificationOutcome::LikelySuccess
                | VerificationOutcome::LikelyFailure
                | VerificationOutcome::Unknown => TaskState::Completed,
            };
            return self.transition(state, None, report.meaning(), now);
        }
        self.transition(
            TaskState::Observing,
            None,
            "Looking at the page again for the next step.",
            now,
        )
    }

    /// Approves the pending bounded action and returns to policy evaluation.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when no task is active, the transition is
    /// illegal, or the approval binding drifted.
    pub fn approve(
        &mut self,
        binding: &ApprovalBinding,
        now: UnixMillis,
    ) -> Result<(ApprovalGrant, Vec<TaskEvent>), OrchestratorError> {
        self.require_active()?;
        let grant = self.approvals.approve(binding, now)?;
        let events = self.transition(
            TaskState::PolicyCheck,
            None,
            "Approved for this exact action. Re-checking the page before it runs.",
            now,
        )?;
        Ok((grant, events))
    }

    /// Cancels the pending approval without approving it.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when no task is active or nothing is
    /// pending.
    pub fn cancel_approval(
        &mut self,
        now: UnixMillis,
    ) -> Result<Vec<TaskEvent>, OrchestratorError> {
        self.require_active()?;
        self.approvals.cancel()?;
        self.transition(
            TaskState::Observing,
            None,
            "The approval request was cancelled. Nothing was executed.",
            now,
        )
    }

    /// Expires a pending approval whose lifetime elapsed and reports it.
    pub fn expire_approvals(&mut self, now: UnixMillis) -> Vec<TaskEvent> {
        let Some(cleared) = self.approvals.expire(now) else {
            return Vec::new();
        };
        debug_assert_eq!(cleared.cause, ClearCause::Expired);
        let detail = "The approval request expired and must be reviewed again.";
        let cleared_approval_action_id = cleared.approval.action_id.clone();
        let mut events = Vec::new();
        if let Some(task) = self.active.as_ref() {
            events.push(build_event(
                &mut self.sequence,
                task,
                EventSpec {
                    state: task.state,
                    wait_reason: task.wait_reason,
                    detail,
                    action_id: Some(cleared_approval_action_id),
                    class: None,
                },
                now,
                self.monotonic_ms,
            ));
        }
        events
    }

    /// Records that an observation trigger invalidated the current view.
    pub fn note_epoch_change(
        &mut self,
        trigger: ba_perception::InvalidationTrigger,
        monotonic_ms: u64,
        now: UnixMillis,
    ) -> Vec<TaskEvent> {
        self.observations.invalidate(trigger, monotonic_ms);
        let Some(cleared) = self.approvals.invalidate(InvalidationReason::PageChanged) else {
            return Vec::new();
        };
        let cleared_action_id = cleared.approval.action_id.clone();
        let detail =
            "The page changed, so the pending approval was cleared. Review the new request.";
        let mut events = Vec::new();
        if let Some(task) = self.active.as_ref() {
            events.push(build_event(
                &mut self.sequence,
                task,
                EventSpec {
                    state: task.state,
                    wait_reason: task.wait_reason,
                    detail,
                    action_id: Some(cleared_action_id),
                    class: None,
                },
                now,
                self.monotonic_ms,
            ));
        }
        events
    }

    /// Pauses the active task.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when no task is active or the transition is
    /// illegal.
    pub fn pause(&mut self, now: UnixMillis) -> Result<Vec<TaskEvent>, OrchestratorError> {
        self.require_active()?;
        self.transition(
            TaskState::Paused,
            None,
            "Paused. No new actions are sent until you resume.",
            now,
        )
    }

    /// Stops dispatch and hands the browser to the user.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when no task is active or the transition is
    /// illegal.
    pub fn take_over(&mut self, now: UnixMillis) -> Result<Vec<TaskEvent>, OrchestratorError> {
        self.require_active()?;
        if let Some(task) = self.active.as_mut() {
            task.stopped_by_user = true;
        }
        self.approvals
            .invalidate(InvalidationReason::UserStoppedAgent);
        self.transition(
            TaskState::UserTakeover,
            None,
            "You're in control. The agent is stopped and pending approvals were cleared.",
            now,
        )
    }

    /// Resumes the task after an explicit user request.
    ///
    /// Resuming always invalidates the previous observation, so the caller must
    /// re-observe and re-run policy before the next action.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when no task is active or the transition is
    /// illegal.
    pub fn resume(&mut self, now: UnixMillis) -> Result<Vec<TaskEvent>, OrchestratorError> {
        self.require_active()?;
        if let Some(task) = self.active.as_mut() {
            task.stopped_by_user = false;
            task.stuck = false;
            task.no_progress_repeats = 0;
            task.verification = None;
        }
        let monotonic_ms = self.monotonic_ms;
        self.observations.invalidate(
            ba_perception::InvalidationTrigger::ActiveTabChanged,
            monotonic_ms,
        );
        self.transition(
            TaskState::Observing,
            None,
            "Resumed. Inspecting the page again before the next step.",
            now,
        )
    }

    /// Cancels the task.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when no task is active or the transition is
    /// illegal.
    pub fn cancel(&mut self, now: UnixMillis) -> Result<Vec<TaskEvent>, OrchestratorError> {
        self.require_active()?;
        if let Some(task) = self.active.as_mut() {
            task.stopped_by_user = true;
        }
        self.approvals
            .invalidate(InvalidationReason::UserStoppedAgent);
        self.transition(
            TaskState::Cancelled,
            None,
            "Task cancelled. An action that was already sent to the page is not undone.",
            now,
        )
    }

    /// Ends the task with the results gathered so far.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when no task is active or the transition is
    /// illegal.
    pub fn finish_with_results(
        &mut self,
        now: UnixMillis,
    ) -> Result<Vec<TaskEvent>, OrchestratorError> {
        self.require_active()?;
        self.transition(
            TaskState::Completed,
            None,
            "Finished with the results gathered so far.",
            now,
        )
    }

    /// Applies the single 50% budget extension to the exhausted dimensions.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError::Budget`] when no extension is available.
    pub fn extend_budget(&mut self, now: UnixMillis) -> Result<Vec<TaskEvent>, OrchestratorError> {
        let exhausted = self
            .active
            .as_ref()
            .ok_or(OrchestratorError::NoActiveTask)?
            .budget
            .exhausted();
        let raised = {
            let task = self
                .active
                .as_mut()
                .ok_or(OrchestratorError::NoActiveTask)?;
            task.budget.extend(&exhausted)?
        };
        let detail = raised
            .iter()
            .map(|(dimension, limit)| format!("{}: {limit}", dimension.plain_language()))
            .collect::<Vec<_>>()
            .join(", ");
        let detail = format!("Budget extended once. New ceilings: {detail}.");
        self.transition(TaskState::Observing, None, &detail, now)
    }

    /// Stops the task for a resource or safety reason, preserving safe state.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestratorError`] when no task is active or the transition is
    /// illegal.
    pub fn hard_stop(
        &mut self,
        detail: &str,
        now: UnixMillis,
    ) -> Result<Vec<TaskEvent>, OrchestratorError> {
        self.require_active()?;
        if let Some(task) = self.active.as_mut() {
            task.verification = Some(VerificationOutcome::Unknown);
            task.stopped_by_user = true;
        }
        let monotonic_ms = self.monotonic_ms;
        self.observations.invalidate(
            ba_perception::InvalidationTrigger::BrowserReconnected,
            monotonic_ms,
        );
        self.approvals.invalidate(InvalidationReason::SystemSuspend);
        let detail =
            format!("{detail} The result of any step that already ran is reported as unknown.");
        self.transition(TaskState::Failed, None, &detail, now)
    }

    /// Advances the orchestrator's monotonic clock, accruing active and waiting
    /// time against the task budget.
    pub fn tick(&mut self, monotonic_ms: u64) {
        let delta = monotonic_ms.saturating_sub(self.monotonic_ms);
        self.monotonic_ms = monotonic_ms;
        if let Some(task) = self.active.as_mut() {
            if task.state.is_active_work() {
                task.budget.accrue_active(delta);
            } else {
                task.budget.accrue_waiting(delta);
            }
        }
    }

    /// Returns the copy for a rejected submission in `requesting_window`.
    ///
    /// The owning window and every other window get different sentences, as
    /// required by BA-WIN-003, and neither implies a hidden queue.
    #[must_use]
    pub fn submission_rejection_copy(&self, requesting_window: &WindowId) -> String {
        match &self.active {
            Some(task) if &task.window == requesting_window => {
                "One Agent task is already running. Finish or cancel it before starting another."
                    .to_owned()
            }
            Some(_) => "Another task is running in a different window.".to_owned(),
            None => String::new(),
        }
    }

    /// Returns the detail of the most recent takeover, when the task is handed
    /// to the user.
    #[must_use]
    pub fn takeover_reason(&self) -> Option<&str> {
        self.active.as_ref().and_then(|task| {
            task.events
                .iter()
                .rev()
                .find(|event| event.state == TaskState::UserTakeover)
                .map(|event| event.detail.as_str())
        })
    }

    fn require_active(&self) -> Result<&ActiveTask, OrchestratorError> {
        self.active.as_ref().ok_or(OrchestratorError::NoActiveTask)
    }

    fn consume(&mut self, dimension: BudgetDimension, amount: u64) -> bool {
        self.active
            .as_mut()
            .is_some_and(|task| task.budget.consume(dimension, amount))
    }

    fn reach_budget_ceiling(
        &mut self,
        now: UnixMillis,
    ) -> Result<Vec<TaskEvent>, OrchestratorError> {
        self.transition(
            TaskState::WaitingForUser,
            Some(WaitReason::TaskBudget),
            "A task budget ceiling was reached. Choose how to continue.",
            now,
        )
    }

    fn transition(
        &mut self,
        to: TaskState,
        wait_reason: Option<WaitReason>,
        detail: &str,
        now: UnixMillis,
    ) -> Result<Vec<TaskEvent>, OrchestratorError> {
        self.transition_with_action(to, wait_reason, detail, None, None, now)
    }

    fn transition_with_action(
        &mut self,
        to: TaskState,
        wait_reason: Option<WaitReason>,
        detail: &str,
        action_id: Option<ActionId>,
        class: Option<ActionClass>,
        now: UnixMillis,
    ) -> Result<Vec<TaskEvent>, OrchestratorError> {
        let monotonic_ms = self.monotonic_ms;
        let terminal;
        let event = {
            let task = self
                .active
                .as_mut()
                .ok_or(OrchestratorError::NoActiveTask)?;
            if task.state != to {
                if task.state.is_terminal() {
                    return Err(OrchestratorError::Transition(TaskStateError::Terminal(
                        task.state,
                    )));
                }
                if !task.state.can_transition_to(to) {
                    return Err(OrchestratorError::Transition(
                        TaskStateError::IllegalTransition {
                            from: task.state,
                            to,
                        },
                    ));
                }
                let waiting = matches!(
                    to,
                    TaskState::WaitingForUser | TaskState::Paused | TaskState::UserTakeover
                );
                if waiting {
                    task.waiting_since_ms = Some(monotonic_ms);
                } else if let Some(since) = task.waiting_since_ms.take() {
                    task.budget
                        .accrue_waiting(monotonic_ms.saturating_sub(since));
                }
                task.state = to;
                task.wait_reason = wait_reason;
                task.revision = task.revision.next().unwrap_or(task.revision);
            }
            terminal = to.is_terminal();
            build_event(
                &mut self.sequence,
                task,
                EventSpec {
                    state: to,
                    wait_reason,
                    detail,
                    action_id,
                    class,
                },
                now,
                monotonic_ms,
            )
        };
        if let Some(task) = self.active.as_mut() {
            task.events.push(event.clone());
        }
        if terminal {
            self.finish_active(now);
        }
        Ok(vec![event])
    }

    fn finish_active(&mut self, now: UnixMillis) {
        let Some(task) = self.active.take() else {
            return;
        };
        let record = TaskRecord {
            task_id: task.id,
            original_text: task.text,
            state: task.state,
            verification: task.verification,
            created_at_unix_ms: task.created_at_unix_ms,
            ended_at_unix_ms: now,
            events: task.events,
            dispatched_actions: task.dispatched_actions,
            extension_used: task.budget.extensions_used > 0,
        };
        self.last_finished = Some(record.clone());
        self.history.push(record);
        self.approvals
            .invalidate(InvalidationReason::UserStoppedAgent);
    }
}

struct EventSpec<'a> {
    state: TaskState,
    wait_reason: Option<WaitReason>,
    detail: &'a str,
    action_id: Option<ActionId>,
    class: Option<ActionClass>,
}

fn build_event(
    sequence: &mut Sequence,
    task: &ActiveTask,
    spec: EventSpec<'_>,
    now: UnixMillis,
    monotonic_ms: u64,
) -> TaskEvent {
    let current = *sequence;
    *sequence = current.next().unwrap_or(current);
    let label = task_state_label(spec.state, spec.wait_reason)
        .unwrap_or("Task ended")
        .to_owned();
    let detail: String = spec.detail.chars().take(512).collect();
    TaskEvent {
        task_id: task.id.clone(),
        revision: task.revision,
        sequence: current,
        state: spec.state,
        wait_reason: spec.wait_reason,
        action_id: spec.action_id,
        action_class: spec.class,
        verification: if spec.state == TaskState::Blocked {
            None
        } else {
            task.verification
        },
        label,
        detail,
        at_unix_ms: now,
        monotonic_ms,
        elapsed_active_ms: task.budget.active_ms.used,
        elapsed_waiting_ms: task.budget.waiting_ms,
        budget: Some(task.budget.view()),
    }
}

fn next_task_id(sequence: &mut Sequence) -> Result<TaskId, OrchestratorError> {
    let current = *sequence;
    *sequence = current.next().ok_or(OrchestratorError::TaskMismatch)?;
    Ok(TaskId::from_number("task", current.get()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ba_domain::{
        Action, BoundedText, BrowserContextId, ClickParams, FrameId, NavigateParams, NodeId,
        ObservationId, TargetRef, TypeParams, ValidatedAction,
    };
    use ba_executor::{AbsentHostPort, Executor, RecordingPort};
    use ba_policy::{
        CapabilityRegistry, DataEgress, EgressProvenance, PolicyContext, ProfileIdentity,
    };

    fn target(label: &str, role: &str) -> TargetRef {
        TargetRef {
            node: NodeId::parse("node-1").unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
            tab: TabId::parse("tab-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            role: Some(BoundedText::parse(role, "target.role", 128).unwrap()),
            label: Some(BoundedText::parse(label, "target.label", 512).unwrap()),
        }
    }

    fn submitting_click() -> ValidatedAction {
        ValidatedAction::new(Action::Click(ClickParams {
            target: target("Place order", "button"),
            opens_new_tab: false,
            may_submit: true,
        }))
        .unwrap()
    }

    fn field_entry() -> ValidatedAction {
        ValidatedAction::new(Action::Type(TypeParams {
            target: target("Card number", "textbox"),
            text: BoundedText::parse("running shoes", "type.text", 4_000).unwrap(),
            submit: false,
        }))
        .unwrap()
    }

    fn now() -> UnixMillis {
        UnixMillis::from_millis(1_700_000_000_000)
    }

    fn window() -> WindowId {
        WindowId::parse("window-1").unwrap()
    }

    fn context(profile: ProfileKind) -> SubmitContext {
        SubmitContext {
            window: window(),
            tab: TabId::parse("tab-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            profile,
        }
    }

    fn policy_context(task_id: &TaskId) -> PolicyContext {
        PolicyContext {
            task_id: task_id.clone(),
            tab: TabId::parse("tab-1").unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            profile: ProfileIdentity {
                context: BrowserContextId::parse("ctx-normal").unwrap(),
                kind: ProfileKind::Normal,
            },
            approved_origins: vec![Origin::parse("https://example.com").unwrap()],
            origin_allowlist_enabled: false,
            audit_healthy: true,
            vault_available: true,
            secure_fill_authorized: false,
        }
    }

    fn navigation(url: &str) -> ValidatedAction {
        ValidatedAction::new(Action::Navigate(NavigateParams {
            url: BoundedText::parse(url, "navigate.url", 2_048).unwrap(),
        }))
        .unwrap()
    }

    fn started() -> (Orchestrator, TaskId) {
        let mut orchestrator = Orchestrator::new();
        let SubmitOutcome::Accepted { task_id, .. } = orchestrator
            .submit("open the pricing page", context(ProfileKind::Normal), now())
            .unwrap()
        else {
            panic!("expected an accepted submission");
        };
        orchestrator.begin_observation(now()).unwrap();
        (orchestrator, task_id)
    }

    #[test]
    fn a_fresh_task_starts_created_then_observes() {
        let mut orchestrator = Orchestrator::new();
        let outcome = orchestrator
            .submit("open the pricing page", context(ProfileKind::Normal), now())
            .unwrap();
        let SubmitOutcome::Accepted { events, .. } = outcome else {
            panic!("expected an accepted submission");
        };
        assert_eq!(events[0].state, TaskState::Created);
        assert_eq!(events[0].label, "Task received");
        assert!(
            !orchestrator.may_dispatch(),
            "a fresh task has not observed the page yet"
        );
        assert_eq!(
            orchestrator.active_origin().map(Origin::canonical),
            Some("https://example.com".to_owned())
        );
        assert_eq!(orchestrator.active_tab().map(TabId::as_str), Some("tab-1"));
        let events = orchestrator.begin_observation(now()).unwrap();
        assert_eq!(events[0].state, TaskState::Normalizing);
        assert_eq!(events[1].state, TaskState::Observing);
        assert_eq!(orchestrator.active_state(), Some(TaskState::Observing));
    }

    #[test]
    fn only_one_task_may_be_active_process_wide() {
        let (mut orchestrator, task_id) = started();
        let error = orchestrator
            .submit("do something else", context(ProfileKind::Normal), now())
            .unwrap_err();
        assert_eq!(
            error,
            OrchestratorError::TaskAlreadyRunning {
                owner_task_id: task_id,
                owner_window: window(),
            }
        );
        assert_eq!(
            orchestrator.submission_rejection_copy(&window()),
            "One Agent task is already running. Finish or cancel it before starting another."
        );
        assert_eq!(
            orchestrator.submission_rejection_copy(&WindowId::parse("window-2").unwrap()),
            "Another task is running in a different window."
        );
    }

    #[test]
    fn private_windows_and_secret_literals_are_refused() {
        let mut orchestrator = Orchestrator::new();
        assert_eq!(
            orchestrator
                .submit("open my bank", context(ProfileKind::Private), now())
                .unwrap_err(),
            OrchestratorError::PrivateWindowAgentDisabled
        );
        let outcome = orchestrator
            .submit(
                "log in with password=hunter2",
                context(ProfileKind::Normal),
                now(),
            )
            .unwrap();
        match outcome {
            SubmitOutcome::NeedsVaultInstead { detail } => {
                assert!(detail.contains("[redacted]"));
                assert!(!detail.contains("hunter2"));
            }
            other => panic!("expected the vault pathway, got {other:?}"),
        }
        assert_eq!(orchestrator.active_state(), None);
    }

    #[test]
    fn empty_and_oversized_instructions_are_refused() {
        let mut orchestrator = Orchestrator::new();
        assert_eq!(
            orchestrator
                .submit("   ", context(ProfileKind::Normal), now())
                .unwrap_err(),
            OrchestratorError::EmptyTask
        );
        let long = "a".repeat(MAX_TASK_TEXT_CODEPOINTS + 1);
        assert_eq!(
            orchestrator
                .submit(&long, context(ProfileKind::Normal), now())
                .unwrap_err(),
            OrchestratorError::TaskTooLong {
                actual: MAX_TASK_TEXT_CODEPOINTS + 1,
                limit: MAX_TASK_TEXT_CODEPOINTS,
            }
        );
    }

    #[test]
    fn a_plain_navigation_is_authorized_then_verified() {
        let (mut orchestrator, task_id) = started();
        let action = navigation("https://example.com/pricing");
        let request = PolicyRequest {
            context: policy_context(&task_id),
            action: &action,
            egress: Vec::new(),
            capability: None,
            destination_requested_by_user: true,
        };
        let proposal = orchestrator.propose(&request, now()).unwrap();
        let ProposalDisposition::Authorized { authorization } = proposal.disposition else {
            panic!("expected an authorization");
        };
        assert_eq!(proposal.class, ActionClass::Read);
        assert_eq!(orchestrator.active_state(), Some(TaskState::Executing));
        assert!(!orchestrator.may_dispatch(), "an action is in flight");

        let mut executor = Executor::new(RecordingPort::accepting(
            ObservationId::parse("epoch-1").unwrap(),
            5,
        ));
        let mut authorization = *authorization;
        let binding = ba_policy::AuthorizationBinding {
            task_id: task_id.clone(),
            tab: TabId::parse("tab-1").unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            digest: action.digest(),
        };
        let dispatch = executor
            .dispatch(&mut authorization, &binding, &action, now())
            .unwrap();
        let events = orchestrator
            .record_dispatch(&dispatch, action.kind(), action.normalized_digest(), now())
            .unwrap();
        assert_eq!(events[0].state, TaskState::Verifying);
        assert_eq!(orchestrator.budget().unwrap().dispatched_actions.used, 1);
        assert_eq!(orchestrator.budget().unwrap().navigations.used, 1);
    }

    #[test]
    fn an_absent_host_results_in_no_page_action_and_a_pause_then_reobserve() {
        let (mut orchestrator, task_id) = started();
        let action = navigation("https://example.com/pricing");
        let request = PolicyRequest {
            context: policy_context(&task_id),
            action: &action,
            egress: Vec::new(),
            capability: None,
            destination_requested_by_user: true,
        };
        let ProposalOutcome {
            disposition: ProposalDisposition::Authorized { authorization },
            ..
        } = orchestrator.propose(&request, now()).unwrap()
        else {
            panic!("expected an authorization");
        };
        let mut executor = Executor::new(AbsentHostPort);
        let mut authorization = *authorization;
        let binding = ba_policy::AuthorizationBinding {
            task_id: TaskId::parse("task-1").unwrap(),
            tab: TabId::parse("tab-1").unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            digest: action.digest(),
        };
        let dispatch = executor
            .dispatch(&mut authorization, &binding, &action, now())
            .unwrap();
        assert!(!dispatch.may_have_reached_page());
        let events = orchestrator
            .record_dispatch(&dispatch, action.kind(), action.normalized_digest(), now())
            .unwrap();
        assert_eq!(events[0].state, TaskState::Verifying);
        assert_eq!(events[1].state, TaskState::Observing);
        assert_eq!(
            orchestrator.budget().unwrap().dispatched_actions.used,
            0,
            "a host that never received input must not consume the action budget"
        );
        assert!(events[0].detail.contains("managed browser host"));
    }

    #[test]
    fn an_irreversible_action_hands_control_to_the_user() {
        let (mut orchestrator, task_id) = started();
        let action = submitting_click();
        let request = PolicyRequest {
            context: policy_context(&task_id),
            action: &action,
            egress: Vec::new(),
            capability: None,
            destination_requested_by_user: true,
        };
        let proposal = orchestrator.propose(&request, now()).unwrap();
        let ProposalDisposition::HandedToUser { presentation } = proposal.disposition else {
            panic!("expected a handoff");
        };
        assert_eq!(proposal.class, ActionClass::Irreversible);
        assert!(presentation.consequences.contains("lasting effect"));
        assert_eq!(orchestrator.active_state(), Some(TaskState::UserTakeover));
        assert!(!orchestrator.may_dispatch());
        assert!(orchestrator.takeover_reason().is_some());

        let events = orchestrator.resume(now()).unwrap();
        assert_eq!(events[0].state, TaskState::Observing);
        assert!(orchestrator.may_dispatch());
    }

    #[test]
    fn an_approvable_write_waits_for_one_decision() {
        let (mut orchestrator, task_id) = started();
        let action = navigation("https://example.com/checkout");
        let request = PolicyRequest {
            context: policy_context(&task_id),
            action: &action,
            egress: vec![DataEgress {
                destination: Origin::parse("https://example.com").unwrap(),
                provenance: EgressProvenance::TaskText,
                bound_origin: None,
                user_authorized: false,
                contains_secret_fingerprint: false,
            }],
            capability: None,
            destination_requested_by_user: false,
        };
        let proposal = orchestrator.propose(&request, now()).unwrap();
        let ProposalDisposition::AwaitingApproval { approval, .. } = proposal.disposition else {
            panic!("expected an approval request");
        };
        assert_eq!(orchestrator.active_state(), Some(TaskState::WaitingForUser));
        let waiting = proposal.events.last().unwrap();
        assert_eq!(waiting.wait_reason, Some(WaitReason::ActionApproval));
        assert_eq!(waiting.label, "Waiting for your approval");
        assert_eq!(waiting.state, TaskState::WaitingForUser);
        assert_eq!(
            approval.expires_at_unix_ms.get(),
            now().get() + 300_000,
            "approval lifetime is five minutes"
        );

        let binding = ApprovalBinding {
            action_id: approval.action_id.clone(),
            digest: action.digest(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
        };
        let (grant, events) = orchestrator.approve(&binding, now()).unwrap();
        assert_eq!(grant.action_id, approval.action_id);
        assert_eq!(events[0].state, TaskState::PolicyCheck);

        // A drifted binding after re-observation must not be approvable.
        let (mut orchestrator, task_id) = started();
        let request = PolicyRequest {
            context: policy_context(&task_id),
            action: &action,
            egress: vec![DataEgress {
                destination: Origin::parse("https://example.com").unwrap(),
                provenance: EgressProvenance::TaskText,
                bound_origin: None,
                user_authorized: false,
                contains_secret_fingerprint: false,
            }],
            capability: None,
            destination_requested_by_user: false,
        };
        let ProposalDisposition::AwaitingApproval { approval, .. } =
            orchestrator.propose(&request, now()).unwrap().disposition
        else {
            panic!("expected an approval request");
        };
        let drifted = ApprovalBinding {
            action_id: approval.action_id.clone(),
            digest: navigation("https://example.com/other").digest(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
        };
        assert!(matches!(
            orchestrator.approve(&drifted, now()),
            Err(OrchestratorError::Approval(ApprovalError::BindingDrift(_)))
        ));
        assert!(orchestrator.pending_approval().is_none());
    }

    #[test]
    fn blocked_policy_never_claims_a_verification_outcome() {
        let (mut orchestrator, task_id) = started();
        let action = field_entry();
        let registry = CapabilityRegistry::builtin();
        let capability = registry.select(ActionKind::Type, "textbox");
        let mut context = policy_context(&task_id);
        context.audit_healthy = false;
        let request = PolicyRequest {
            context,
            action: &action,
            egress: Vec::new(),
            capability,
            destination_requested_by_user: true,
        };
        let proposal = orchestrator.propose(&request, now()).unwrap();
        let ProposalDisposition::Refused { rule, .. } = proposal.disposition else {
            panic!("expected a refusal");
        };
        assert_eq!(rule, RuleId::AuditUnavailable);
        assert_eq!(orchestrator.active_state(), None);
        let record = orchestrator.last_finished().unwrap();
        assert_eq!(record.state, TaskState::Blocked);
        assert!(record.verification.is_none());
        assert!(
            record
                .events
                .iter()
                .all(|event| event.verification.is_none())
        );
    }

    #[test]
    fn the_budget_ceiling_waits_for_the_user_and_can_be_extended_once() {
        let (mut orchestrator, task_id) = started();
        let action = navigation("https://example.com/page");
        let context = policy_context(&task_id);
        let mut executor = Executor::new(RecordingPort::accepting(
            ObservationId::parse("epoch-1").unwrap(),
            5,
        ));
        let report = ba_verify::evaluate(&[], &ba_verify::Evidence::default());
        for _ in 0..61 {
            let request = PolicyRequest {
                context: context.clone(),
                action: &action,
                egress: Vec::new(),
                capability: None,
                destination_requested_by_user: true,
            };
            let proposal = orchestrator.propose(&request, now()).unwrap();
            if matches!(proposal.disposition, ProposalDisposition::BudgetReached) {
                assert_eq!(orchestrator.active_state(), Some(TaskState::WaitingForUser));
                let waiting = proposal.events.last().unwrap();
                assert_eq!(waiting.wait_reason, Some(WaitReason::TaskBudget));
                assert_eq!(waiting.label, "Task budget reached");
                let events = orchestrator.extend_budget(now()).unwrap();
                assert!(events[0].detail.contains("Budget extended once"));
                assert_eq!(
                    orchestrator.budget().unwrap().action_proposals.limit,
                    90,
                    "the ceiling rises by at most 50%"
                );
                assert!(matches!(
                    orchestrator.extend_budget(now()),
                    Err(OrchestratorError::Budget(BudgetError::AlreadyExtended))
                ));
                return;
            }
            let ProposalDisposition::Authorized { authorization } = proposal.disposition else {
                panic!("expected an authorization");
            };
            let mut authorization = *authorization;
            let binding = ba_policy::AuthorizationBinding {
                task_id: task_id.clone(),
                tab: TabId::parse("tab-1").unwrap(),
                frame: FrameId::parse("frame-1").unwrap(),
                origin: Origin::parse("https://example.com").unwrap(),
                epoch: ObservationId::parse("epoch-1").unwrap(),
                digest: action.digest(),
            };
            let dispatch = executor
                .dispatch(&mut authorization, &binding, &action, now())
                .unwrap();
            orchestrator
                .record_dispatch(&dispatch, action.kind(), action.normalized_digest(), now())
                .unwrap();
            orchestrator
                .record_verification(&report, false, now())
                .unwrap();
            orchestrator.note_semantic_change();
        }
        panic!("the proposal ceiling was never reached");
    }

    #[test]
    fn two_identical_actions_without_progress_pause_the_task() {
        let (mut orchestrator, task_id) = started();
        let action = navigation("https://example.com/page");
        let context = policy_context(&task_id);
        let request = PolicyRequest {
            context,
            action: &action,
            egress: Vec::new(),
            capability: None,
            destination_requested_by_user: true,
        };
        let mut executor = Executor::new(RecordingPort::accepting(
            ObservationId::parse("epoch-1").unwrap(),
            5,
        ));
        // No page change is ever reported between the two identical steps, which
        // is exactly the no-progress condition in BA-STUCK-001.
        let report = ba_verify::evaluate(
            &[ba_verify::Postcondition::TextContains {
                expected: "unreachable".to_owned(),
            }],
            &ba_verify::Evidence {
                fresh: true,
                ..ba_verify::Evidence::default()
            },
        );
        let mut events = Vec::new();
        let mut observed: Vec<Option<TaskState>> = Vec::new();
        for _ in 0..2 {
            let proposal = orchestrator.propose(&request, now()).unwrap();
            let ProposalDisposition::Authorized { authorization } = proposal.disposition else {
                panic!("expected an authorization, got {:?}", proposal.disposition);
            };
            let mut authorization = *authorization;
            let binding = ba_policy::AuthorizationBinding {
                task_id: task_id.clone(),
                tab: TabId::parse("tab-1").unwrap(),
                frame: FrameId::parse("frame-1").unwrap(),
                origin: Origin::parse("https://example.com").unwrap(),
                epoch: ObservationId::parse("epoch-1").unwrap(),
                digest: action.digest(),
            };
            let dispatch = executor
                .dispatch(&mut authorization, &binding, &action, now())
                .unwrap();
            orchestrator
                .record_dispatch(&dispatch, action.kind(), action.normalized_digest(), now())
                .unwrap();
            observed.push(orchestrator.active_state());
            events = orchestrator
                .record_verification(&report, false, now())
                .unwrap();
            observed.push(orchestrator.active_state());
            if orchestrator.active_state() == Some(TaskState::Paused) {
                break;
            }
        }
        assert_eq!(
            orchestrator.active_state(),
            Some(TaskState::Paused),
            "a third identical action must not be dispatched (observed {observed:?})"
        );
        let stuck_event = events.last().unwrap();
        assert_eq!(stuck_event.state, TaskState::Paused);
        assert!(stuck_event.detail.contains("appears stuck"));
        assert!(!orchestrator.may_dispatch());
        assert_eq!(
            orchestration_stuck_controls(),
            vec![
                "Retry observation".to_owned(),
                "Take over".to_owned(),
                "Cancel task".to_owned()
            ]
        );
    }

    fn orchestration_stuck_controls() -> Vec<String> {
        Orchestrator::stuck_controls()
            .iter()
            .map(|control| (*control).to_owned())
            .collect()
    }

    #[test]
    fn cancellation_is_not_a_failure_and_keeps_the_result_unknown() {
        let (mut orchestrator, _) = started();
        let events = orchestrator.cancel(now()).unwrap();
        assert_eq!(events[0].state, TaskState::Cancelled);
        assert_eq!(events[0].label, "Task cancelled");
        let record = orchestrator.last_finished().unwrap();
        assert_eq!(record.state, TaskState::Cancelled);
        assert_eq!(record.verification, None);
        assert!(record.summary_line().contains("Verification not run"));
    }

    #[test]
    fn a_hard_stop_fails_the_task_and_reports_unknown() {
        let (mut orchestrator, _) = started();
        let events = orchestrator
            .hard_stop("Measured memory pressure reached the safe limit.", now())
            .unwrap();
        assert_eq!(events[0].state, TaskState::Failed);
        let record = orchestrator.last_finished().unwrap();
        assert_eq!(record.verification, Some(VerificationOutcome::Unknown));
        assert_eq!(record.state, TaskState::Failed);
    }

    #[test]
    fn terminal_states_are_never_mutated() {
        let (mut orchestrator, _) = started();
        orchestrator.cancel(now()).unwrap();
        assert!(matches!(
            orchestrator.pause(now()),
            Err(OrchestratorError::NoActiveTask)
        ));
    }

    #[test]
    fn clarification_is_limited_to_two_rounds() {
        let (mut orchestrator, _) = started();
        let first = orchestrator
            .ask_clarification("Which account should I use?", now())
            .unwrap();
        assert_eq!(first[0].wait_reason, Some(WaitReason::UserInput));
        assert_eq!(first[0].label, "Needs your input");
        orchestrator.resume(now()).unwrap();
        orchestrator
            .ask_clarification("And which folder?", now())
            .unwrap();
        orchestrator.resume(now()).unwrap();
        assert!(matches!(
            orchestrator.ask_clarification("And the date?", now()),
            Err(OrchestratorError::ClarificationLimitReached)
        ));
    }

    #[test]
    fn a_page_change_clears_a_pending_approval() {
        let (mut orchestrator, task_id) = started();
        let action = navigation("https://example.com/checkout");
        let request = PolicyRequest {
            context: policy_context(&task_id),
            action: &action,
            egress: vec![DataEgress {
                destination: Origin::parse("https://example.com").unwrap(),
                provenance: EgressProvenance::TaskText,
                bound_origin: None,
                user_authorized: false,
                contains_secret_fingerprint: false,
            }],
            capability: None,
            destination_requested_by_user: false,
        };
        orchestrator.propose(&request, now()).unwrap();
        assert!(orchestrator.pending_approval().is_some());
        let events = orchestrator.note_epoch_change(
            ba_perception::InvalidationTrigger::DomMutation,
            500,
            now(),
        );
        assert_eq!(events.len(), 1);
        assert!(events[0].detail.contains("pending approval was cleared"));
        assert!(orchestrator.pending_approval().is_none());
    }

    #[test]
    fn expiry_is_reported_without_executing_anything() {
        let (mut orchestrator, task_id) = started();
        let action = navigation("https://example.com/checkout");
        let request = PolicyRequest {
            context: policy_context(&task_id),
            action: &action,
            egress: vec![DataEgress {
                destination: Origin::parse("https://example.com").unwrap(),
                provenance: EgressProvenance::TaskText,
                bound_origin: None,
                user_authorized: false,
                contains_secret_fingerprint: false,
            }],
            capability: None,
            destination_requested_by_user: false,
        };
        orchestrator.propose(&request, now()).unwrap();
        let events = orchestrator.expire_approvals(UnixMillis::from_millis(now().get() + 300_000));
        assert_eq!(events.len(), 1);
        assert!(events[0].detail.contains("expired"));
        assert!(orchestrator.pending_approval().is_none());
    }

    #[test]
    fn a_task_mismatch_is_refused() {
        let (mut orchestrator, _) = started();
        let action = navigation("https://example.com/page");
        let request = PolicyRequest {
            context: policy_context(&TaskId::parse("task-999").unwrap()),
            action: &action,
            egress: Vec::new(),
            capability: None,
            destination_requested_by_user: true,
        };
        assert_eq!(
            orchestrator.propose(&request, now()).unwrap_err(),
            OrchestratorError::TaskMismatch
        );
    }

    #[test]
    fn active_time_and_waiting_time_are_tracked_separately() {
        let (mut orchestrator, _) = started();
        orchestrator.tick(1_000);
        orchestrator.pause(now()).unwrap();
        orchestrator.tick(6_000);
        let budget = orchestrator.budget().unwrap();
        assert_eq!(budget.active_ms.used, 1_000);
        assert_eq!(budget.waiting_ms, 5_000);
    }

    #[test]
    fn finished_tasks_keep_a_masked_history_entry() {
        let mut orchestrator = Orchestrator::new();
        let SubmitOutcome::Accepted { .. } = orchestrator
            .submit(
                "find the cheapest shoes",
                context(ProfileKind::Normal),
                now(),
            )
            .unwrap()
        else {
            panic!("expected an accepted submission");
        };
        orchestrator.begin_observation(now()).unwrap();
        orchestrator.cancel(now()).unwrap();
        let history = orchestrator.history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].prefill_text(), "find the cheapest shoes");
        assert!(history[0].summary_line().contains("CANCELLED"));
        assert!(history[0].summary_line().contains("Verification not run"));
    }

    #[test]
    fn copy_contract_is_available_from_the_orchestrator() {
        let contract = Orchestrator::copy_contract();
        assert!(contract.contains("Performing action"));
        assert!(contract.contains("error_copy"));
    }

    #[test]
    fn readiness_copy_is_shared_with_the_panel() {
        let (label, controls) = Orchestrator::readiness_copy(ReadinessState::Parked);
        assert_eq!(label, "Model paused to free memory");
        assert_eq!(controls, &["Resume model"]);
    }

    #[test]
    fn invalid_transitions_are_refused_rather_than_coerced() {
        let mut orchestrator = Orchestrator::new();
        let SubmitOutcome::Accepted { .. } = orchestrator
            .submit("open the pricing page", context(ProfileKind::Normal), now())
            .unwrap()
        else {
            panic!("expected an accepted submission");
        };
        // CREATED cannot jump straight to EXECUTING.
        assert!(matches!(
            orchestrator.transition(TaskState::Executing, None, "x", now()),
            Err(OrchestratorError::Transition(
                TaskStateError::IllegalTransition {
                    from: TaskState::Created,
                    to: TaskState::Executing,
                }
            ))
        ));
    }
}
