// Task Runtime.
//
// The orchestrator-owned execution loop: perception -> reasoning -> policy ->
// execution -> verification. This module connects the existing subsystems
// into one coherent runtime flow and owns all task lifecycle state.
//
// C-01: 5 engines -> orchestrator-owned progress -> semantic state graph.
// C-154: orchestrator-owned typed task-progress object updated
//        deterministically after each verified action.
// C-63: hard stop = preserve safe state -> notify -> require intervention;
//        resumption never reuses stale state.
// C-64: Phase-1 slice uses a deterministic proposer isolated from release
//        behavior until the local inference path is attached.
// C-141: actions are bound to the epoch from which they were derived.
// C-25: execution-time recheck; epoch change invalidates approval and
//        requires one fresh approval.
// C-12/handoff: no autonomous action is dispatched while human control is
//        active.
//
// Ownership decisions landed here:
//  - The runtime owns task state, step index, epoch, mode, authorization
//    state, pending intervention, and last verification outcome. The
//    frontend only projects these through typed events.
//  - Cancellation is a real flag checked at step boundaries; the abort
//    command never relies on frontend state.
//  - The cancellation/abort path emits the terminal task state from Rust.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use tauri::AppHandle;

use crate::core::action::commitment::{action_commitment_hash, ActionCommitment};
use crate::core::execution::primitives::{self, ExecutionInput, ExecutionResult};
use crate::core::orchestrator::budgets::BudgetPolicy;
use crate::core::orchestrator::progress::{ProgressMode, TaskProgress, TaskState};
use crate::core::orchestrator::state_machine::{StateMachine, TransitionError};
use crate::core::perception::frames::FrameObservation;
use crate::core::perception::graph::GraphBounds;
use crate::core::perception::pipeline::{self, PipelineOutput};
use crate::core::policy::classes::SideEffectClass;
use crate::core::policy::confirmation::ConfirmationPolicy;
use crate::core::policy::tiers::AuthorizationTier;
use crate::core::reasoning::schema::ModelAction;
use crate::error::Error;
use crate::ipc::{
    emit_action_durable_state, emit_action_proposal, emit_agent_cursor_state,
    emit_authorization_required, emit_authorization_resolved, emit_execution_result,
    emit_human_takeover_state, emit_perception_state, emit_policy_decision, emit_task_state,
    emit_verification_outcome,
};
use crate::security::clocks::MonotonicClock;

/// Maximum consecutive perception/reasoning retries before the runtime
/// stops with an explicit reason instead of spinning.
const MAX_CONSECUTIVE_RETRIES: u32 = 3;

/// Outcome of a single execution step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepOutcome {
    /// The step completed an action with an independent verification result.
    ActionCompleted {
        action: String,
        success: bool,
        verification: String,
    },
    /// Execution attempted but failed; the task may continue.
    ActionFailed { action: String, reason: String },
    /// Policy denied the proposed action; the task has stopped.
    Denied { action: String, reason: String },
    /// Policy requires user authorization before execution; loop pauses.
    AwaitingConfirmation { action: String, tier: String },
    /// Human control is active; autonomous execution is paused.
    WaitingForHuman { reason: String },
    /// The runtime stopped at a terminal boundary.
    Stopped { reason: String },
    /// Transient failure; retry after a short delay.
    Retry { reason: String },
}

/// A live task runtime plus its cancellation flag.
///
/// The registry owns slots; the loop thread and IPC commands share them.
pub struct TaskSlot {
    pub task_id: String,
    pub runtime: Mutex<TaskRuntime>,
    pub cancel: Arc<AtomicBool>,
}

impl TaskSlot {
    pub fn new(runtime: TaskRuntime) -> Self {
        let task_id = runtime.task_id.clone();
        Self {
            task_id,
            runtime: Mutex::new(runtime),
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}

/// Registry of active task runtimes, managed by Tauri state.
///
/// Commands look up a slot by task id to abort, confirm, deny, begin or
/// end human handoff. The frontend never reaches into this structure
/// directly; it only sends task-identifying requests.
#[derive(Default)]
pub struct TaskRegistry {
    tasks: Mutex<HashMap<String, Arc<TaskSlot>>>,
}

impl TaskRegistry {
    pub fn insert(&self, task_id: String, slot: Arc<TaskSlot>) {
        self.tasks.lock().insert(task_id, slot);
    }

    pub fn get(&self, task_id: &str) -> Option<Arc<TaskSlot>> {
        self.tasks.lock().get(task_id).cloned()
    }

    pub fn remove(&self, task_id: &str) {
        self.tasks.lock().remove(task_id);
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.lock().is_empty()
    }

    pub fn len(&self) -> usize {
        self.tasks.lock().len()
    }
}

/// The orchestrator-owned task runtime that drives the execution loop.
pub struct TaskRuntime {
    task_id: String,
    state: TaskState,
    step_index: u32,
    total_steps: u32,
    epoch: u64,
    mode: ProgressMode,
    budget: BudgetPolicy,
    max_confirmations: Option<u32>,
    app: AppHandle,
    /// Active action commitment awaiting or holding user authorization.
    pending_commitment: Option<ActionCommitment>,
    /// Authorization capability issued after user confirmation.
    pending_capability: Option<crate::core::action::ActionAuthorizationCapability>,
    /// The proposal the pending commitment is bound to.
    pending_proposal: Option<ModelAction>,
    /// Confirmation count against the MAX_CONFIRMATIONS budget.
    confirmations_used: u32,
    /// Whether human control of the managed browser is active.
    handoff_active: bool,
    /// Whether the pending commitment has been confirmed by the user.
    execution_confirmed: bool,
    /// Policy verdict recorded for the current proposal.
    current_policy_verdict: Option<String>,
    /// Policy reason recorded for the current proposal.
    current_policy_reason: Option<String>,
    /// Authorization tier derived by policy for the current proposal.
    current_policy_tier: Option<AuthorizationTier>,
    /// Last independent verification outcome, used for terminal state.
    last_verification: Option<String>,
}

impl TaskRuntime {
    pub fn new(
        task_id: String,
        app: AppHandle,
        config: crate::config::RuntimeConfig,
    ) -> Result<Self, Error> {
        Ok(Self {
            task_id,
            state: TaskState::Running,
            step_index: 1,
            total_steps: 11,
            epoch: 1,
            mode: ProgressMode::ModelReasoning,
            max_confirmations: config.max_confirmations,
            budget: BudgetPolicy::new(config),
            app,
            pending_commitment: None,
            pending_capability: None,
            pending_proposal: None,
            confirmations_used: 0,
            handoff_active: false,
            execution_confirmed: false,
            current_policy_verdict: None,
            current_policy_reason: None,
            current_policy_tier: None,
            last_verification: None,
        })
    }

    /// Run one step of the execution loop.
    ///
    /// Flow: (resume pending authorization | perception -> reasoning ->
    /// policy -> execution -> verification), each phase emitting typed
    /// IPC events the frontend projects.
    pub fn step(&mut self) -> Result<StepOutcome, Error> {
        // No autonomous work while human control is active (handoff).
        if self.handoff_active {
            return Ok(StepOutcome::WaitingForHuman {
                reason: "human control active; autonomous execution paused".into(),
            });
        }

        // A confirmed commitment resumes directly into the execution-time
        // recheck path; the confirmed action is not re-proposed.
        if self.execution_confirmed {
            return self.resume_after_confirmation();
        }

        // Budget gate before every step (C-149 -> C-63 stop semantics).
        let snapshot = self
            .budget
            .snapshot(self.step_index, 0, 0, self.confirmations_used);
        if let Some(exhausted) = self.budget.any_exhausted(&snapshot) {
            let reason = format!("STOPPED: BUDGET EXHAUSTED ({exhausted})");
            self.stop_with_reason(&reason)?;
            return Ok(StepOutcome::Stopped { reason });
        }

        // Phase 1: perception.
        let perception = match self.capture_perception() {
            Ok(p) => p,
            Err(e) => {
                return Ok(StepOutcome::Retry {
                    reason: format!("perception unavailable: {e}"),
                });
            }
        };

        // Phase 2: reasoning (deterministic proposer until inference attaches).
        let proposal = match self.reason(&perception) {
            Ok(p) => p,
            Err(e) => {
                return Ok(StepOutcome::Retry {
                    reason: format!("reasoning unavailable: {e}"),
                });
            }
        };

        // Phase 3: policy decision.
        let (verdict, tier) = match self.evaluate_policy(&proposal, &perception) {
            Ok(v) => v,
            Err(e) => {
                let reason = format!("STOPPED: POLICY EVALUATION FAILED ({e})");
                self.stop_with_reason(&reason)?;
                return Ok(StepOutcome::Stopped { reason });
            }
        };

        // Phase 4: execution or authorization request.
        match verdict.as_str() {
            "ALLOW" => match self.execute_action(&proposal, &perception, tier) {
                Ok(result) => {
                    // Phase 5: independent verification.
                    let verification = self.verify_result(&proposal, &result, &perception)?;
                    self.last_verification = Some(verification.clone());
                    self.advance_step();
                    Ok(StepOutcome::ActionCompleted {
                        action: proposal_action_label(&proposal),
                        success: result.success,
                        verification,
                    })
                }
                Err(e) => {
                    self.last_verification = Some("UNKNOWN".into());
                    self.advance_step();
                    Ok(StepOutcome::ActionFailed {
                        action: proposal_action_label(&proposal),
                        reason: e.to_string(),
                    })
                }
            },
            "DENY" => {
                let reason = self
                    .current_policy_reason()
                    .unwrap_or_else(|| "policy denied the action".into());
                self.transition_state(TaskState::Aborted)?;
                Ok(StepOutcome::Denied {
                    action: proposal_action_label(&proposal),
                    reason,
                })
            }
            "CONFIRMATION_REQUIRED" => {
                let tier_str = tier_to_string(tier);
                let commitment = self.build_commitment(&proposal, tier);
                let commitment_hash =
                    hex32(&action_commitment_hash(&commitment.canonical_for_hashing()));
                let summary = format!(
                    "{} at epoch {} requires approval",
                    proposal_action_label(&proposal),
                    commitment.state_epoch
                );
                self.pending_proposal = Some(proposal.clone());
                self.pending_commitment = Some(commitment);
                self.pending_capability = None;
                self.execution_confirmed = false;
                emit_authorization_required(
                    &self.app,
                    crate::ipc::events::AuthorizationRequiredEvent {
                        task_id: self.task_id.clone(),
                        tier: tier_str.clone(),
                        summary,
                        commitment_hash,
                    },
                )?;
                Ok(StepOutcome::AwaitingConfirmation {
                    action: proposal_action_label(&proposal),
                    tier: tier_str,
                })
            }
            other => {
                let reason = format!("STOPPED: UNKNOWN POLICY VERDICT ({other})");
                self.stop_with_reason(&reason)?;
                Ok(StepOutcome::Stopped { reason })
            }
        }
    }

    /// Execution-time path for a user-confirmed action (C-25 recheck).
    ///
    /// Revalidates the commitment against the current epoch before any
    /// dispatch. On epoch mismatch the approval is invalidated and exactly
    /// one fresh approval is requested.
    fn resume_after_confirmation(&mut self) -> Result<StepOutcome, Error> {
        if self.handoff_active {
            return Ok(StepOutcome::WaitingForHuman {
                reason: "human control active; confirmed action deferred".into(),
            });
        }

        let commitment = self.pending_commitment.clone().ok_or_else(|| {
            Error::StateMismatch("confirmed action has no pending commitment".into())
        })?;
        let proposal = self
            .pending_proposal
            .clone()
            .ok_or_else(|| Error::StateMismatch("confirmed action has no proposal".into()))?;
        let capability = self
            .pending_capability
            .clone()
            .ok_or_else(|| Error::StateMismatch("authorization capability missing".into()))?;

        // C-25: execution-time recheck. A state change since approval
        // invalidates the approval; request one fresh approval.
        if commitment.state_epoch != self.epoch {
            let tier = self
                .current_policy_tier
                .unwrap_or(AuthorizationTier::NativeConfirm);
            let rebuilt = self.build_commitment(&proposal, tier);
            let commitment_hash = hex32(&action_commitment_hash(&rebuilt.canonical_for_hashing()));
            let tier_str = tier_to_string(tier);
            self.pending_commitment = Some(rebuilt);
            self.pending_capability = None;
            self.execution_confirmed = false;
            emit_authorization_required(
                &self.app,
                crate::ipc::events::AuthorizationRequiredEvent {
                    task_id: self.task_id.clone(),
                    tier: tier_str.clone(),
                    summary: format!(
                        "state changed since approval; fresh approval required for {}",
                        proposal_action_label(&proposal)
                    ),
                    commitment_hash,
                },
            )?;
            return Ok(StepOutcome::AwaitingConfirmation {
                action: proposal_action_label(&proposal),
                tier: tier_str,
            });
        }

        if !crate::core::action::authz::verify_action_authorization_capability(
            &capability,
            &commitment,
            &capability.authorization_nonce,
            MonotonicClock::now_nanos(),
        ) {
            self.clear_pending_authorization();
            return Err(Error::StateMismatch(
                "authorization expired or does not match current commitment".into(),
            ));
        }

        // Fresh perception at execution time.
        let perception = match self.capture_perception() {
            Ok(p) => p,
            Err(e) => {
                return Ok(StepOutcome::Retry {
                    reason: format!("execution-time perception unavailable: {e}"),
                });
            }
        };

        let tier = self
            .current_policy_tier
            .unwrap_or(AuthorizationTier::NativeConfirm);
        let outcome = match self.execute_action(&proposal, &perception, tier) {
            Ok(result) => {
                let verification = self.verify_result(&proposal, &result, &perception)?;
                self.last_verification = Some(verification.clone());
                self.advance_step();
                StepOutcome::ActionCompleted {
                    action: proposal_action_label(&proposal),
                    success: result.success,
                    verification,
                }
            }
            Err(e) => {
                self.last_verification = Some("UNKNOWN".into());
                self.advance_step();
                StepOutcome::ActionFailed {
                    action: proposal_action_label(&proposal),
                    reason: e.to_string(),
                }
            }
        };
        self.clear_pending_authorization();
        Ok(outcome)
    }

    fn clear_pending_authorization(&mut self) {
        self.pending_commitment = None;
        self.pending_capability = None;
        self.pending_proposal = None;
        self.execution_confirmed = false;
    }

    /// Capture perception from the managed browser runtime.
    fn capture_perception(&self) -> Result<PipelineOutput, Error> {
        let frames = self.frame_observation()?;
        let caps = GraphBounds::new();
        let output = pipeline::build(&frames, self.epoch, self.task_id.clone(), &caps)
            .map_err(|e| Error::Internal(format!("perception pipeline: {e:?}")))?;
        emit_perception_state(
            &self.app,
            crate::ipc::events::PerceptionStateEvent {
                task_id: self.task_id.clone(),
                epoch: self.epoch,
                frame_id: output.epoch.frame_id.clone(),
                loader_id: output.epoch.loader_id.clone(),
                actionable_count: output.actionable_count,
                bounded_exceeded: output.bounded_exceeded,
            },
        )?;
        Ok(output)
    }

    /// Frame observation source.
    ///
    /// When the managed runtime is unavailable this fails closed with
    /// BROWSER_RUNTIME_UNAVAILABLE; no synthetic perception is emitted to
    /// the frontend. The CDP-backed capture path attaches here once the
    /// target/session layer is live; until then only a real attached
    /// runtime can produce observations.
    fn frame_observation(&self) -> Result<Vec<FrameObservation>, Error> {
        use tauri::Manager;
        if let Some(ctrl) = self
            .app
            .try_state::<Arc<crate::browser::controller::BrowserRuntimeController>>()
        {
            if ctrl.snapshot().available() {
                return ctrl.capture_frame_observations(&self.task_id);
            }
        }
        Err(Error::Unsupported(
            "BROWSER_RUNTIME_UNAVAILABLE: CDP observation capture not attached".into(),
        ))
    }

    /// Produce a structured action proposal for the current perception.
    ///
    /// Phase-1 deterministic proposer. It is structurally isolated from the
    /// release inference path: when the verified local model attaches, this
    /// function is replaced by the inference call (same input contract:
    /// normalized goal + bounded semantic state + TaskProgress + policy
    /// context).
    fn reason(&mut self, perception: &PipelineOutput) -> Result<ModelAction, Error> {
        // C-154: progress object accompanies every proposal.
        let _progress = TaskProgress {
            task_id: self.task_id.clone(),
            current_state: self.state.clone(),
            step_index: self.step_index,
            total_steps: self.total_steps,
            epoch: self.epoch,
            mode: self.mode.clone(),
        };

        let action = if perception.actionable_count == 0 {
            ModelAction::Wait {
                reason: "no actionable nodes in bounded state".into(),
            }
        } else {
            match self.step_index % 4 {
                1 => ModelAction::Click {
                    semantic_reference: "button:Submit".into(),
                },
                2 => ModelAction::Navigate {
                    url: "https://example.test/page".into(),
                },
                3 => ModelAction::Type {
                    semantic_reference: "field:search".into(),
                    text: "test query".into(),
                },
                _ => ModelAction::Wait {
                    reason: "observing page state".into(),
                },
            }
        };

        self.pending_proposal = Some(action.clone());

        emit_action_proposal(
            &self.app,
            crate::ipc::events::ActionProposalEvent {
                task_id: self.task_id.clone(),
                action: proposal_action_label(&action),
                action_class: class_label(action.action_class()),
                semantic_reference: action_semantic_ref(&action),
                epoch: self.epoch,
            },
        )?;
        emit_agent_cursor_state(
            &self.app,
            crate::ipc::events::AgentCursorStateEvent {
                active: true,
                state: "MOVING".into(),
                target_frame_id: Some("main".into()),
                target_loader_id: Some(format!("L-{}", self.epoch)),
                x: None,
                y: None,
                reason: Some(format!("proposing: {}", proposal_action_label(&action))),
            },
        )?;
        Ok(action)
    }

    /// Evaluate the proposal through Policy and emit the decision.
    ///
    /// Tier derivation here is the conservative Phase-1 mapping; the
    /// catalog HIGH_STAKES threshold is undefined and is not invented, so
    /// no action is classified BIOMETRIC_CONFIRM by default.
    fn evaluate_policy(
        &mut self,
        action: &ModelAction,
        _perception: &PipelineOutput,
    ) -> Result<(String, AuthorizationTier), Error> {
        let action_class = action.action_class();
        let tier = match action_class.effective_class() {
            SideEffectClass::Read => AuthorizationTier::None,
            SideEffectClass::ReversibleWrite => AuthorizationTier::PolicyOnly,
            SideEffectClass::Irreversible | SideEffectClass::Unknown => {
                AuthorizationTier::NativeConfirm
            }
        };

        let (verdict, reason) = policy_verdict(
            action_class,
            tier,
            self.confirmations_used,
            self.max_confirmations,
        );
        let reason = reason.unwrap_or_else(|| "policy decision recorded".into());

        self.current_policy_verdict = Some(verdict.to_string());
        self.current_policy_reason = Some(reason.clone());
        self.current_policy_tier = Some(tier);

        emit_policy_decision(
            &self.app,
            crate::ipc::events::PolicyDecisionEvent {
                task_id: self.task_id.clone(),
                action_class: class_label(action_class),
                tier: tier_to_string(tier),
                verdict: verdict.to_string(),
                reason,
            },
        )?;
        emit_agent_cursor_state(
            &self.app,
            crate::ipc::events::AgentCursorStateEvent {
                active: true,
                state: match verdict {
                    "ALLOW" => "CLICKING",
                    "DENY" => "BLOCKED",
                    _ => "MOVING",
                }
                .to_string(),
                target_frame_id: Some("main".into()),
                target_loader_id: Some(format!("L-{}", self.epoch)),
                x: None,
                y: None,
                reason: Some(format!("policy: {verdict}")),
            },
        )?;

        Ok((verdict.to_string(), tier))
    }

    /// Execute the proposed action through the typed execution engine.
    fn execute_action(
        &mut self,
        action: &ModelAction,
        perception: &PipelineOutput,
        tier: AuthorizationTier,
    ) -> Result<ExecutionResult, Error> {
        // Capability verification before any dispatch (C-36/C-39/C-100).
        if tier != AuthorizationTier::None {
            if let (Some(cap), Some(commitment)) =
                (&self.pending_capability, &self.pending_commitment)
            {
                if !crate::core::action::authz::verify_action_authorization_capability(
                    cap,
                    commitment,
                    &cap.authorization_nonce,
                    MonotonicClock::now_nanos(),
                ) {
                    return Err(Error::StateMismatch(
                        "authorization does not match current commitment/epoch".into(),
                    ));
                }
            } else {
                return Err(Error::PolicyBlocked(
                    "side-effectful action lacks authorization capability".into(),
                ));
            }
        }

        let input = ExecutionInput {
            action: action.clone(),
            epoch: self.epoch,
            target_frame_id: perception.epoch.frame_id.clone(),
            target_loader_id: perception.epoch.loader_id.clone(),
            policy_tier: tier,
            confirmation_commitment_id: self
                .pending_commitment
                .as_ref()
                .map(|c| hex32(&action_commitment_hash(&c.canonical_for_hashing()))),
        };

        let result = primitives::ExecutionEngine::execute(input)?;

        emit_execution_result(
            &self.app,
            crate::ipc::events::ExecutionResultEvent {
                task_id: self.task_id.clone(),
                action: proposal_action_label(action),
                success: result.success,
                failure_reason: result.failure_reason.clone(),
                post_state: result.post_state.clone(),
                epoch: self.epoch,
            },
        )?;
        emit_agent_cursor_state(
            &self.app,
            crate::ipc::events::AgentCursorStateEvent {
                active: result.success,
                state: if result.success { "IDLE" } else { "BLOCKED" }.into(),
                target_frame_id: Some(perception.epoch.frame_id.clone()),
                target_loader_id: Some(perception.epoch.loader_id.clone()),
                x: None,
                y: None,
                reason: Some(
                    result
                        .failure_reason
                        .clone()
                        .unwrap_or_else(|| "execution dispatched".into()),
                ),
            },
        )?;
        Ok(result)
    }

    /// Verify the execution result independently of the model (C-158).
    fn verify_result(
        &self,
        action: &ModelAction,
        result: &ExecutionResult,
        _perception: &PipelineOutput,
    ) -> Result<String, Error> {
        // Phase-1 verification: execution status plus action class. The
        // full independent-evidence verification engine attaches here; no
        // claim stronger than the evidence is emitted.
        let outcome = if result.success {
            match action.action_class().effective_class() {
                SideEffectClass::Read => "VERIFIED_SUCCESS",
                _ => "LIKELY_SUCCESS",
            }
        } else {
            "VERIFIED_FAILURE"
        }
        .to_string();

        emit_verification_outcome(
            &self.app,
            crate::ipc::events::VerificationOutcomeEvent {
                task_id: self.task_id.clone(),
                outcome: outcome.clone(),
                evidence_count: 1,
                timestamp: MonotonicClock::now_nanos().to_string(),
            },
        )?;
        Ok(outcome)
    }

    /// Advance step and epoch (each step re-observes from a fresh epoch).
    fn advance_step(&mut self) {
        self.step_index = self.step_index.saturating_add(1);
        self.epoch = self.epoch.saturating_add(1);
    }

    /// Transition the task state through the state machine and emit it.
    fn transition_state(&mut self, new_state: TaskState) -> Result<(), Error> {
        let current = self.state.clone();
        match StateMachine::transition(current.clone(), new_state.clone()) {
            Ok(next) => {
                self.state = next;
                emit_task_state(
                    &self.app,
                    &self.task_id,
                    state_to_task_status(&self.state),
                    &format!("{} / {}", self.step_index, self.total_steps),
                )?;
                Ok(())
            }
            Err(TransitionError::NotAllowed) => Err(Error::StateMismatch(format!(
                "invalid transition from {:?} to {:?}",
                current, new_state
            ))),
            Err(_) => Err(Error::StateMismatch("invalid transition".into())),
        }
    }

    /// Emit an explicit terminal stop state with the backend reason.
    fn stop_with_reason(&mut self, reason: &str) -> Result<(), Error> {
        self.state = TaskState::Failed;
        emit_task_state(
            &self.app,
            &self.task_id,
            crate::ipc::events::TaskStatus::Failed,
            reason,
        )
    }

    /// Emit the terminal state derived from the last verification outcome.
    pub(crate) fn complete(&mut self) -> Result<(), Error> {
        let target = match self.last_verification.as_deref() {
            Some("VERIFIED_SUCCESS") => TaskState::Verified,
            Some("LIKELY_SUCCESS") => TaskState::LikelySuccess,
            Some("VERIFIED_FAILURE") | Some("LIKELY_FAILURE") => TaskState::Failed,
            _ => TaskState::Unknown,
        };
        self.transition_state(target)
    }

    fn current_policy_reason(&self) -> Option<String> {
        self.current_policy_reason.clone()
    }

    /// Build the canonical commitment for the proposed action.
    fn build_commitment(&self, action: &ModelAction, _tier: AuthorizationTier) -> ActionCommitment {
        let destination = match action {
            ModelAction::Navigate { url } => url.clone(),
            ModelAction::Click { semantic_reference } => format!("click:{semantic_reference}"),
            ModelAction::Type {
                semantic_reference, ..
            } => {
                format!("type:{semantic_reference}")
            }
            ModelAction::Select {
                semantic_reference, ..
            } => {
                format!("select:{semantic_reference}")
            }
            ModelAction::Scroll { direction, .. } => format!("scroll:{}", scroll_label(direction)),
            ModelAction::Wait { .. } => "wait".into(),
            ModelAction::PressKey { key } => format!("key:{key}"),
            ModelAction::SecureFill { field_reference } => {
                format!("secure-fill:{field_reference}")
            }
            ModelAction::RequestConfirmation => "request-confirmation".into(),
        };

        ActionCommitment::new(
            self.task_id.clone(),
            None,
            "runtime:managed".into(),
            destination,
            self.epoch,
            300_000,
            MonotonicClock::now_nanos(),
        )
    }

    /// Validate and record a user confirmation for the pending commitment.
    ///
    /// The frontend only echoes the backend-issued commitment hash; the
    /// backend generates the nonce, revalidates the commitment, and issues
    /// the capability. The frontend never authors the facts being
    /// authorized.
    pub fn confirm_authorization(&mut self, commitment_hash_hex: &str) -> Result<(), Error> {
        let commitment = self
            .pending_commitment
            .clone()
            .ok_or_else(|| Error::InvalidParameter("no pending authorization".into()))?;

        let expected = hex32(&action_commitment_hash(&commitment.canonical_for_hashing()));
        if expected != commitment_hash_hex {
            return Err(Error::StateMismatch(
                "commitment hash does not match pending authorization".into(),
            ));
        }

        let nonce: [u8; 32] = rand::random();
        let capability = crate::core::action::ActionAuthorizationCapability::new(
            &commitment,
            commitment.operation_type,
            nonce,
            MonotonicClock::now_nanos(),
            300_000,
        );

        self.pending_capability = Some(capability);
        self.execution_confirmed = true;
        self.confirmations_used = self.confirmations_used.saturating_add(1);

        emit_authorization_resolved(
            &self.app,
            crate::ipc::events::AuthorizationResolvedEvent {
                task_id: self.task_id.clone(),
                decision: "APPROVED".into(),
            },
        )?;
        emit_action_durable_state(
            &self.app,
            crate::ipc::events::ActionDurableStateEvent {
                task_id: self.task_id.clone(),
                durable_id: format!("da-{}", self.task_id),
                durable_state: "READY".into(),
                action_commitment_hash: expected,
            },
        )?;
        Ok(())
    }

    /// Deny the pending authorization and stop the task.
    pub fn deny_authorization(&mut self) -> Result<(), Error> {
        if self.pending_commitment.is_none() {
            return Err(Error::InvalidParameter("no pending authorization".into()));
        }
        self.clear_pending_authorization();
        emit_authorization_resolved(
            &self.app,
            crate::ipc::events::AuthorizationResolvedEvent {
                task_id: self.task_id.clone(),
                decision: "DENIED".into(),
            },
        )?;
        if self.state == TaskState::Running {
            self.transition_state(TaskState::Aborted)?;
        }
        Ok(())
    }

    /// Begin human control of the managed browser.
    ///
    /// Autonomous dispatch is paused for the task lifetime of the handoff;
    /// the takeover state is backend-owned.
    pub fn begin_human_handoff(&mut self) -> Result<String, Error> {
        if self.handoff_active {
            return Err(Error::StateMismatch("human control already active".into()));
        }
        self.handoff_active = true;
        self.mode = ProgressMode::Intervention;
        let durable_id = format!("dh-{}", self.task_id);

        emit_task_state(
            &self.app,
            &self.task_id,
            crate::ipc::events::TaskStatus::Parked,
            &format!(
                "{} / {} · HANDED TO USER",
                self.step_index, self.total_steps
            ),
        )?;
        emit_human_takeover_state(
            &self.app,
            crate::ipc::events::HumanTakeoverStateEvent {
                task_id: self.task_id.clone(),
                mode: "HANDOFF_ACTIVE".into(),
                reason: Some("agent paused; user controls the managed browser".into()),
            },
        )?;
        emit_action_durable_state(
            &self.app,
            crate::ipc::events::ActionDurableStateEvent {
                task_id: self.task_id.clone(),
                durable_id: durable_id.clone(),
                durable_state: "WAITING_FOR_EXTERNAL_AUTH".into(),
                action_commitment_hash: format!("0x{}", hex32(&[0u8; 32])),
            },
        )?;
        Ok(durable_id)
    }

    /// Return control to the agent after human takeover.
    ///
    /// Advances the state epoch so any approval or pending action bound to
    /// the pre-handoff epoch is invalidated; the next step re-perceives.
    pub fn end_human_handoff(&mut self) -> Result<(), Error> {
        if !self.handoff_active {
            return Err(Error::StateMismatch("no active human control".into()));
        }
        self.handoff_active = false;
        self.mode = ProgressMode::ModelReasoning;
        // Fresh observation, new epoch (C-25: stale approvals invalidated).
        self.epoch = self.epoch.saturating_add(1);
        // Any confirmed-but-unexecuted action from before the handoff is
        // invalidated by the epoch advance; it must be re-approved.
        if self.execution_confirmed && self.pending_commitment.is_some() {
            self.pending_capability = None;
            self.execution_confirmed = false;
        }

        emit_human_takeover_state(
            &self.app,
            crate::ipc::events::HumanTakeoverStateEvent {
                task_id: self.task_id.clone(),
                mode: "IDLE".into(),
                reason: Some(format!(
                    "control returned; fresh perception at epoch {}",
                    self.epoch
                )),
            },
        )?;
        emit_task_state(
            &self.app,
            &self.task_id,
            crate::ipc::events::TaskStatus::Running,
            &format!("{} / {}", self.step_index, self.total_steps),
        )?;
        Ok(())
    }

    /// Abort the task at a defined boundary and emit the terminal state.
    pub fn abort(&mut self) -> Result<(), Error> {
        if matches!(self.state, TaskState::Aborted | TaskState::Failed) {
            return Ok(());
        }
        self.clear_pending_authorization();
        self.handoff_active = false;
        if self.state == TaskState::Running {
            self.transition_state(TaskState::Aborted)
        } else {
            self.state = TaskState::Aborted;
            emit_task_state(
                &self.app,
                &self.task_id,
                crate::ipc::events::TaskStatus::Aborted,
                &format!("{} / {}", self.step_index, self.total_steps),
            )
        }
    }

    pub(crate) fn step_index(&self) -> u32 {
        self.step_index
    }

    pub(crate) fn total_steps(&self) -> u32 {
        self.total_steps
    }
}

/// Spawn the background execution loop for a task slot.
///
/// The loop checks the cancellation flag at step boundaries and exits on
/// terminal, awaiting-confirmation, or human-control outcomes.
pub fn spawn_loop(registry: Arc<TaskRegistry>, slot: Arc<TaskSlot>) -> Result<(), Error> {
    std::thread::Builder::new()
        .name(format!("task-loop-{}", slot.task_id))
        .spawn(move || run_loop(&registry, &slot))
        .map(|_| ())
        .map_err(|e| Error::Internal(format!("failed to spawn task loop: {e}")))
}

fn run_loop(registry: &TaskRegistry, slot: &Arc<TaskSlot>) {
    let mut consecutive_retries: u32 = 0;
    loop {
        if slot.cancel.load(Ordering::SeqCst) {
            // Abort command already emitted the terminal state.
            registry.remove(&slot.task_id);
            return;
        }

        let outcome = {
            let mut runtime = slot.runtime.lock();
            runtime.step()
        };

        match outcome {
            Ok(StepOutcome::Retry { reason }) => {
                consecutive_retries += 1;
                if consecutive_retries >= MAX_CONSECUTIVE_RETRIES {
                    let mut runtime = slot.runtime.lock();
                    let stop = format!("STOPPED: {reason}");
                    let _ = runtime.stop_with_reason(&stop);
                    registry.remove(&slot.task_id);
                    return;
                }
                std::thread::sleep(Duration::from_millis(250));
            }
            Ok(StepOutcome::ActionCompleted { .. }) | Ok(StepOutcome::ActionFailed { .. }) => {
                consecutive_retries = 0;
                let done = {
                    let runtime = slot.runtime.lock();
                    runtime.step_index() > runtime.total_steps()
                };
                if done {
                    let mut runtime = slot.runtime.lock();
                    let _ = runtime.complete();
                    registry.remove(&slot.task_id);
                    return;
                }
            }
            Ok(StepOutcome::Denied { .. }) | Ok(StepOutcome::Stopped { .. }) => {
                registry.remove(&slot.task_id);
                return;
            }
            Ok(StepOutcome::AwaitingConfirmation { .. })
            | Ok(StepOutcome::WaitingForHuman { .. }) => {
                // Pause; the loop respawns after confirm or handoff return.
                return;
            }
            Err(_) => {
                let mut runtime = slot.runtime.lock();
                let _ = runtime.stop_with_reason("STOPPED: INTERNAL RUNTIME ERROR");
                registry.remove(&slot.task_id);
                return;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (unit-tested; wire-format contract lives here).
// ---------------------------------------------------------------------------

/// Policy verdict derivation, extracted for testing.
pub(crate) fn policy_verdict(
    action_class: SideEffectClass,
    tier: AuthorizationTier,
    confirmations_used: u32,
    max_confirmations: Option<u32>,
) -> (&'static str, Option<String>) {
    let requirement = ConfirmationPolicy::requirement(tier, action_class);
    if !requirement.required {
        (
            "ALLOW",
            Some(
                requirement
                    .reason
                    .unwrap_or_else(|| "read-only policy action".into()),
            ),
        )
    } else if max_confirmations
        .map(|max| confirmations_used >= max)
        .unwrap_or(false)
    {
        (
            "DENY",
            Some("MAX_CONFIRMATIONS budget exhausted; task stops".into()),
        )
    } else {
        (
            "CONFIRMATION_REQUIRED",
            Some(
                requirement
                    .reason
                    .unwrap_or_else(|| "trusted confirmation is required".into()),
            ),
        )
    }
}

/// Wire-format labels for authorization tiers (SCREAMING_SNAKE_CASE).
pub(crate) fn tier_to_string(tier: AuthorizationTier) -> String {
    match tier {
        AuthorizationTier::None => "NONE",
        AuthorizationTier::PolicyOnly => "POLICY_ONLY",
        AuthorizationTier::NativeConfirm => "NATIVE_CONFIRM",
        AuthorizationTier::BiometricConfirm => "BIOMETRIC_CONFIRM",
    }
    .to_string()
}

/// Wire-format labels for action classes (C-06 vocabulary).
pub(crate) fn class_label(class: SideEffectClass) -> String {
    match class {
        SideEffectClass::Read => "READ",
        SideEffectClass::ReversibleWrite => "REVERSIBLE_WRITE",
        SideEffectClass::Irreversible => "IRREVERSIBLE",
        SideEffectClass::Unknown => "UNKNOWN",
    }
    .to_string()
}

/// Map orchestrator task state to the IPC task status enum.
pub(crate) fn state_to_task_status(state: &TaskState) -> crate::ipc::events::TaskStatus {
    match state {
        TaskState::Pending => crate::ipc::events::TaskStatus::Pending,
        TaskState::Running => crate::ipc::events::TaskStatus::Running,
        TaskState::Verified => crate::ipc::events::TaskStatus::Verified,
        TaskState::LikelySuccess => crate::ipc::events::TaskStatus::LikelySuccess,
        TaskState::Unknown => crate::ipc::events::TaskStatus::Unknown,
        TaskState::Failed => crate::ipc::events::TaskStatus::Failed,
        TaskState::Aborted => crate::ipc::events::TaskStatus::Aborted,
    }
}

/// Ledger label for a proposed action. Typed text content is never logged;
/// only the target reference is.
pub(crate) fn proposal_action_label(action: &ModelAction) -> String {
    match action {
        ModelAction::Navigate { .. } => "NAVIGATE".into(),
        ModelAction::Click { semantic_reference } => format!("CLICK {semantic_reference}"),
        ModelAction::Type {
            semantic_reference, ..
        } => format!("TYPE {semantic_reference}"),
        ModelAction::Select {
            semantic_reference, ..
        } => {
            format!("SELECT {semantic_reference}")
        }
        ModelAction::Scroll { direction, .. } => format!("SCROLL {}", scroll_label(direction)),
        ModelAction::Wait { .. } => "WAIT".into(),
        ModelAction::PressKey { key } => format!("PRESS {key}"),
        ModelAction::SecureFill { field_reference } => {
            format!("SECURE_FILL {field_reference}")
        }
        ModelAction::RequestConfirmation => "REQUEST_CONFIRMATION".into(),
    }
}

fn scroll_label(direction: &crate::core::reasoning::schema::ScrollDirection) -> &'static str {
    use crate::core::reasoning::schema::ScrollDirection;
    match direction {
        ScrollDirection::Up => "UP",
        ScrollDirection::Down => "DOWN",
        ScrollDirection::Left => "LEFT",
        ScrollDirection::Right => "RIGHT",
    }
}

/// Semantic reference attached to an action proposal event, when the
/// action targets a specific node.
pub(crate) fn action_semantic_ref(action: &ModelAction) -> Option<String> {
    match action {
        ModelAction::Click { semantic_reference }
        | ModelAction::Type {
            semantic_reference, ..
        }
        | ModelAction::Select {
            semantic_reference, ..
        } => Some(semantic_reference.clone()),
        ModelAction::SecureFill { field_reference } => Some(field_reference.clone()),
        _ => None,
    }
}

/// Lowercase hex encoding for 32-byte digests (commitment hashes).
pub(crate) fn hex32(bytes: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(64);
    for b in bytes {
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_labels_match_wire_contract() {
        assert_eq!(tier_to_string(AuthorizationTier::None), "NONE");
        assert_eq!(tier_to_string(AuthorizationTier::PolicyOnly), "POLICY_ONLY");
        assert_eq!(
            tier_to_string(AuthorizationTier::NativeConfirm),
            "NATIVE_CONFIRM"
        );
        assert_eq!(
            tier_to_string(AuthorizationTier::BiometricConfirm),
            "BIOMETRIC_CONFIRM"
        );
    }

    #[test]
    fn action_class_labels_match_c06_vocabulary() {
        assert_eq!(class_label(SideEffectClass::Read), "READ");
        assert_eq!(
            class_label(SideEffectClass::ReversibleWrite),
            "REVERSIBLE_WRITE"
        );
        assert_eq!(class_label(SideEffectClass::Irreversible), "IRREVERSIBLE");
        assert_eq!(class_label(SideEffectClass::Unknown), "UNKNOWN");
    }

    #[test]
    fn read_action_under_none_tier_is_allowed() {
        let (verdict, reason) =
            policy_verdict(SideEffectClass::Read, AuthorizationTier::None, 0, None);
        assert_eq!(verdict, "ALLOW");
        assert!(reason.is_some());
    }

    #[test]
    fn side_effectful_action_requires_confirmation() {
        let (verdict, _) = policy_verdict(
            SideEffectClass::ReversibleWrite,
            AuthorizationTier::PolicyOnly,
            0,
            None,
        );
        assert_eq!(verdict, "CONFIRMATION_REQUIRED");
    }

    #[test]
    fn exhausted_confirmation_budget_denies() {
        let (verdict, reason) = policy_verdict(
            SideEffectClass::Irreversible,
            AuthorizationTier::NativeConfirm,
            2,
            Some(2),
        );
        assert_eq!(verdict, "DENY");
        assert!(reason.unwrap().contains("MAX_CONFIRMATIONS"));
    }

    #[test]
    fn undefined_confirmation_budget_is_not_invented() {
        let (verdict, _) = policy_verdict(
            SideEffectClass::Irreversible,
            AuthorizationTier::NativeConfirm,
            u32::MAX,
            None,
        );
        assert_eq!(verdict, "CONFIRMATION_REQUIRED");
    }

    #[test]
    fn unknown_class_is_effectively_irreversible_for_confirmation() {
        let (verdict, _) = policy_verdict(
            SideEffectClass::Unknown,
            AuthorizationTier::NativeConfirm,
            0,
            None,
        );
        assert_eq!(verdict, "CONFIRMATION_REQUIRED");
    }

    #[test]
    fn state_to_status_mapping_is_total() {
        use crate::ipc::events::TaskStatus;
        assert!(matches!(
            state_to_task_status(&TaskState::Pending),
            TaskStatus::Pending
        ));
        assert!(matches!(
            state_to_task_status(&TaskState::Running),
            TaskStatus::Running
        ));
        assert!(matches!(
            state_to_task_status(&TaskState::Verified),
            TaskStatus::Verified
        ));
        assert!(matches!(
            state_to_task_status(&TaskState::LikelySuccess),
            TaskStatus::LikelySuccess
        ));
        assert!(matches!(
            state_to_task_status(&TaskState::Unknown),
            TaskStatus::Unknown
        ));
        assert!(matches!(
            state_to_task_status(&TaskState::Failed),
            TaskStatus::Failed
        ));
        assert!(matches!(
            state_to_task_status(&TaskState::Aborted),
            TaskStatus::Aborted
        ));
    }

    #[test]
    fn action_labels_never_contain_typed_text() {
        let action = ModelAction::Type {
            semantic_reference: "field:search".into(),
            text: "sensitive typed content".into(),
        };
        let label = proposal_action_label(&action);
        assert_eq!(label, "TYPE field:search");
        assert!(!label.contains("sensitive"));
    }

    #[test]
    fn secure_fill_label_carries_only_field_reference() {
        let action = ModelAction::SecureFill {
            field_reference: "field:password".into(),
        };
        assert_eq!(proposal_action_label(&action), "SECURE_FILL field:password");
        assert_eq!(
            action_semantic_ref(&action).as_deref(),
            Some("field:password")
        );
    }

    #[test]
    fn hex32_is_lowercase_and_full_width() {
        let mut bytes = [0u8; 32];
        bytes[0] = 0xAB;
        bytes[31] = 0x0F;
        let hex = hex32(&bytes);
        assert_eq!(hex.len(), 64);
        assert!(hex.starts_with("ab"));
        assert!(hex.ends_with("0f"));
        assert!(hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn commitment_hash_hex_round_trip_is_deterministic() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "runtime:managed".into(),
            "click:button:Submit".into(),
            7,
            300_000,
            1_000_000,
        );
        let a = hex32(&action_commitment_hash(&commitment.canonical_for_hashing()));
        let b = hex32(&action_commitment_hash(&commitment.canonical_for_hashing()));
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn different_epoch_produces_different_commitment_hash() {
        let at_epoch = |epoch| {
            let c = ActionCommitment::new(
                "task-1".into(),
                None,
                "runtime:managed".into(),
                "click:button:Submit".into(),
                epoch,
                300_000,
                1_000_000,
            );
            hex32(&action_commitment_hash(&c.canonical_for_hashing()))
        };
        assert_ne!(at_epoch(1), at_epoch(2));
    }

    #[test]
    fn registry_add_get_remove() {
        // Registry mechanics without a Tauri app handle.
        let registry = TaskRegistry::default();
        assert_eq!(registry.len(), 0);

        // A slot requires a live runtime (AppHandle), so only verify the
        // empty-lookup contract here.
        assert!(registry.get("missing").is_none());
        registry.remove("missing");
        assert_eq!(registry.len(), 0);
    }
}
