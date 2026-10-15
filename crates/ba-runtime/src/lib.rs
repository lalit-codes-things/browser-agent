//! Browser-Agent application core (Sections 10.3, 10.4, 20.2).
//!
//! The core wires the trusted layers together and translates typed UI commands
//! into authoritative runtime events. It is the only place that knows both the
//! IPC contract and the orchestrator.
//!
//! Two channels are deliberately distinct:
//!
//! * the **command channel** carries first-party UI intent over IPC and is
//!   treated as untrusted input;
//! * the **host channel** carries state reported by the managed CEF adapter
//!   ([`AppCore::report_browser_state`], [`AppCore::report_dispatch_result`]).
//!   It is a native contract that the UI cannot reach, which is why the tab
//!   origin used for policy comes from the core's own browser state and never
//!   from a UI payload.
//!
//! When no managed host is attached, browser and window commands report a
//! truthful four-part error instead of simulating a page.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![deny(missing_docs)]

use ba_domain::{
    ActionKind, ErrorCase, FrameId, ObservationId, Origin, ReadinessState, Sequence, TabId, TaskId,
    UnixMillis, VerificationOutcome, WindowId,
};
use ba_executor::DispatchOutcome;
use ba_ipc::{
    ApprovalView, BrowserStateView, Command, DataCommand, DownloadView, EventPayload, HandoffView,
    ModelCommand, NoteView, SequenceTracker, SkillView, TaskHistoryEntry, TaskStatusView,
    WindowCommand,
};
use ba_model::{MeasuredProgress, ModelManifest, ReadinessMachine};
use ba_orchestrator::{
    Orchestrator, OrchestratorError, ProposalDisposition, SubmitContext, SubmitOutcome, TaskRecord,
};
use ba_policy::{ApprovalBinding, PolicyContext, PolicyRequest, ProfileIdentity, RuleId};
use ba_verify::VerificationReport;
use serde::Serialize;

/// Core failures that produce a user-visible error event.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RuntimeError {
    /// An orchestrator operation failed.
    #[error(transparent)]
    Orchestrator(#[from] OrchestratorError),
    /// The requested operation needs the managed host.
    #[error("the managed browser host is not attached")]
    HostUnavailable,
}

/// A local research note held by the core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    /// Identity.
    pub note: ba_domain::NoteId,
    /// Title.
    pub title: String,
    /// Body.
    pub body: String,
    /// Source URL.
    pub source_url: String,
    /// Capture time.
    pub captured_at_unix_ms: UnixMillis,
}

/// A reviewed skill held by the core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    /// Identity.
    pub skill: ba_domain::SkillId,
    /// Display name.
    pub name: String,
    /// Immutable version label.
    pub version: String,
    /// Provenance.
    pub provenance: String,
    /// Whether it is enabled.
    pub enabled: bool,
}

/// One outbound event with its assigned sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundEvent {
    /// Assigned sequence.
    pub sequence: Sequence,
    /// Payload.
    pub payload: EventPayload,
    /// Task identity, when the payload belongs to one.
    pub task_id: Option<TaskId>,
}

/// The application core.
pub struct AppCore {
    orchestrator: Orchestrator,
    readiness: ReadinessMachine,
    browser: Option<BrowserStateView>,
    notes: Vec<Note>,
    skills: Vec<Skill>,
    downloads: Vec<DownloadView>,
    history_mirror: Vec<TaskHistoryEntry>,
    manifests: Vec<ModelManifest>,
    audit_healthy: bool,
    audit_detail: String,
    out_sequence: Sequence,
    monotonic_ms: u64,
}

impl Default for AppCore {
    fn default() -> Self {
        Self::new()
    }
}

impl AppCore {
    /// Creates a core with no managed host attached.
    #[must_use]
    pub fn new() -> Self {
        Self {
            orchestrator: Orchestrator::new(),
            readiness: ReadinessMachine::new(),
            browser: None,
            notes: Vec::new(),
            skills: Vec::new(),
            downloads: Vec::new(),
            history_mirror: Vec::new(),
            manifests: Vec::new(),
            audit_healthy: true,
            audit_detail: "Integrity verified (no records yet)".to_owned(),
            out_sequence: Sequence::new(1),
            monotonic_ms: 0,
        }
    }

    /// Returns the copy contract JSON shared by the runtime and the UI.
    #[must_use]
    pub fn copy_contract() -> String {
        Orchestrator::copy_contract()
    }

    /// Borrows the orchestrator, for the host adapter.
    #[must_use]
    pub const fn orchestrator(&self) -> &Orchestrator {
        &self.orchestrator
    }

    /// Returns the current model readiness state.
    #[must_use]
    pub const fn readiness_state(&self) -> ReadinessState {
        self.readiness.state()
    }

    /// Registers a frozen model manifest with the core.
    pub fn register_manifest(&mut self, manifest: ModelManifest) {
        self.manifests.push(manifest);
    }

    /// Registers a reviewed skill with the core.
    pub fn register_skill(&mut self, skill: Skill) {
        self.skills.push(skill);
    }

    /// Sets the audit health reported by the audit layer.
    pub fn set_audit_health(&mut self, healthy: bool, detail: &str) {
        self.audit_healthy = healthy;
        self.audit_detail = detail.to_owned();
    }

    /// Reports a frame the core refused before it could be handled.
    ///
    /// The rejection is emitted as a normal outbound event, in the same
    /// sequence space as every other event, so the interface receives one
    /// well-formed envelope stream and can render the reason. A bare payload or
    /// a separate counter would break that stream.
    pub fn reject_frame(&mut self, command: &str, reason: &str, now: UnixMillis) -> OutboundEvent {
        self.push(
            EventPayload::CommandRejected {
                command: command.to_owned(),
                reason: reason.to_owned(),
            },
            None,
            now,
        )
    }

    /// Advances the monotonic clock.
    pub fn tick(&mut self, monotonic_ms: u64) {
        self.monotonic_ms = monotonic_ms;
        self.orchestrator.tick(monotonic_ms);
    }

    /// Reports browser state from the managed host.
    ///
    /// This is the native host contract: the first-party UI cannot call it, so
    /// the origin used by policy is always the host's own report.
    pub fn report_browser_state(&mut self, state: Option<BrowserStateView>) -> Vec<EventPayload> {
        self.browser = state.clone();
        vec![EventPayload::BrowserState {
            state: state.map(Box::new),
        }]
    }

    /// Records the observation step for a freshly accepted task.
    ///
    /// The perception layer calls this once it has produced the first semantic
    /// state for the task's tab. Until it runs, the task stays in `CREATED` and
    /// no proposal can be evaluated, so an approval or handoff can only ever be
    /// reached from a real observation.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when no task is active or the transition is
    /// illegal.
    pub fn begin_observation(
        &mut self,
        now: UnixMillis,
    ) -> Result<Vec<OutboundEvent>, RuntimeError> {
        let events = self.orchestrator.begin_observation(now)?;
        Ok(self.wrap_task_events(events))
    }

    /// Reports the executor result for the in-flight action from the host.
    pub fn report_dispatch_result(
        &mut self,
        outcome: &DispatchOutcome,
        kind: ActionKind,
        normalized_digest: ba_domain::ActionDigest,
        now: UnixMillis,
    ) -> Result<Vec<OutboundEvent>, RuntimeError> {
        let events = self
            .orchestrator
            .record_dispatch(outcome, kind, normalized_digest, now)?;
        Ok(self.wrap_task_events(events))
    }

    /// Reports a verification result from the verifier.
    pub fn report_verification(
        &mut self,
        report: &VerificationReport,
        terminal: bool,
        now: UnixMillis,
    ) -> Result<Vec<OutboundEvent>, RuntimeError> {
        let events = self
            .orchestrator
            .record_verification(report, terminal, now)?;
        Ok(self.wrap_task_events(events))
    }

    /// Handles one UI command, returning the events it produced.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] only when the core itself cannot continue; user
    /// errors such as a missing host are returned as `Error` events so the UI can
    /// render the four-part contract.
    pub fn handle(&mut self, command: &Command, now: UnixMillis) -> Vec<OutboundEvent> {
        match command {
            Command::Browser(_) | Command::Window(WindowCommand::Close(_)) => {
                self.emit_error(ErrorCase::BrowserHostUnavailable.copy(), None, now)
            }
            Command::Window(_) => {
                self.emit_error(ErrorCase::BrowserHostUnavailable.copy(), None, now)
            }
            Command::Agent(agent) => self.handle_agent_command(agent, now),
            Command::Model(model) => self.handle_model_command(model, now),
            Command::Data(data) => self.handle_data_command(data, now),
        }
    }

    fn handle_agent_command(
        &mut self,
        command: &ba_ipc::AgentCommand,
        now: UnixMillis,
    ) -> Vec<OutboundEvent> {
        use ba_ipc::AgentCommand;
        match command {
            AgentCommand::SubmitTask(args) => {
                self.submit_task(&args.text, args.window.clone(), args.tab.clone(), now)
            }
            AgentCommand::Pause(_) => {
                let result = self.orchestrator.pause(now);
                self.orchestrator_event(result, now)
            }
            AgentCommand::Resume(_) => {
                let result = self.orchestrator.resume(now);
                self.orchestrator_event(result, now)
            }
            AgentCommand::CancelTask(_) => {
                let result = self.orchestrator.cancel(now);
                self.orchestrator_event(result, now)
            }
            AgentCommand::TakeOver(_) => {
                let result = self.orchestrator.take_over(now);
                self.orchestrator_event(result, now)
            }
            AgentCommand::ExtendBudget(_) => {
                let result = self.orchestrator.extend_budget(now);
                self.orchestrator_event(result, now)
            }
            AgentCommand::FinishWithResults(_) => {
                let result = self.orchestrator.finish_with_results(now);
                self.orchestrator_event(result, now)
            }
            AgentCommand::CancelApproval(_) => {
                let result = self.orchestrator.cancel_approval(now);
                self.orchestrator_event(result, now)
            }
            AgentCommand::ApproveAction(args) => {
                let binding = ApprovalBinding {
                    action_id: args.action_id.clone(),
                    digest: args.digest,
                    origin: args.origin.clone(),
                    epoch: args.epoch.clone(),
                    frame: args.frame.clone(),
                };
                match self.orchestrator.approve(&binding, now) {
                    Ok((grant, events)) => {
                        let mut out = self.wrap_task_events(events);
                        if grant.granted_at_unix_ms.get() <= now.get() {
                            let task_id = self.orchestrator.active_task_id().cloned();
                            out.push(self.push(
                                EventPayload::ApprovalCleared {
                                    cause: "APPROVED".to_owned(),
                                    detail:
                                        "Approved for this exact action. The page is checked again before it runs."
                                            .to_owned(),
                                },
                                task_id,
                                now,
                            ));
                        }
                        out
                    }
                    Err(error) => self.emit_orchestrator_error(&error, now),
                }
            }
        }
    }

    fn handle_model_command(
        &mut self,
        command: &ModelCommand,
        now: UnixMillis,
    ) -> Vec<OutboundEvent> {
        match command {
            ModelCommand::Status(_) => vec![self.readiness_event(now)],
            ModelCommand::Install(args) => {
                let known = self
                    .manifests
                    .iter()
                    .any(|manifest| manifest.id == args.manifest_id);
                if !known {
                    return self.emit_error(
                        ErrorCase::ModelNotReady.copy(),
                        Some(format!(
                            "No frozen, integrity-checked artifact named `{}` is registered in this build, so nothing was downloaded.",
                            args.manifest_id
                        )),
                        now,
                    );
                }
                match self.readiness.start_install() {
                    Ok(()) => vec![self.readiness_event(now)],
                    Err(_) => self.emit_error(ErrorCase::ModelNotReady.copy(), None, now),
                }
            }
            ModelCommand::CancelInstall(_) => match self.readiness.cancel_install() {
                Ok(()) => vec![self.readiness_event(now)],
                Err(_) => self.emit_error(ErrorCase::ModelNotReady.copy(), None, now),
            },
            ModelCommand::Verify(_) => {
                vec![self.readiness_event(now)]
            }
            ModelCommand::Resume(_) => {
                if matches!(
                    self.readiness.state(),
                    ReadinessState::Parked | ReadinessState::RunnerFailed
                ) {
                    if self.readiness.mark_ready().is_ok() {
                        vec![self.readiness_event(now)]
                    } else {
                        self.emit_error(ErrorCase::ModelNotReady.copy(), None, now)
                    }
                } else {
                    self.emit_error(
                        ErrorCase::ModelNotReady.copy(),
                        Some("The model is not parked, so there is nothing to resume.".to_owned()),
                        now,
                    )
                }
            }
        }
    }

    fn handle_data_command(
        &mut self,
        command: &DataCommand,
        now: UnixMillis,
    ) -> Vec<OutboundEvent> {
        match command {
            DataCommand::ForgetSite(_) | DataCommand::ClearBrowsingData(_) => self.emit_error(
                ErrorCase::BrowserHostUnavailable.copy(),
                Some(
                    "Site data lives in the managed browser profile, which is not attached, so nothing was deleted."
                        .to_owned(),
                ),
                now,
            ),
            DataCommand::ClearTaskHistory(_) => {
                self.history_mirror.clear();
                vec![self.push(
                    EventPayload::TaskHistory {
                        entries: Vec::new(),
                    },
                    None,
                    now,
                )]
            }
            DataCommand::ListNotes(_) => vec![self.notes_event(now)],
            DataCommand::CreateNote(args) => {
                let sequence = self.out_sequence;
                self.out_sequence = sequence.next().unwrap_or(sequence);
                self.notes.push(Note {
                    note: ba_domain::NoteId::from_number("note", sequence.get()),
                    title: args.title.clone(),
                    body: args.body.clone(),
                    source_url: args.source_url.clone(),
                    captured_at_unix_ms: now,
                });
                vec![self.notes_event(now)]
            }
            DataCommand::DeleteNote(args) => {
                self.notes.retain(|note| note.note != args.note);
                vec![self.notes_event(now)]
            }
            DataCommand::ListSkills(_) => vec![self.skills_event(now)],
            DataCommand::RevokeSkill(args) => {
                for skill in &mut self.skills {
                    if skill.skill == args.skill {
                        skill.enabled = false;
                    }
                }
                vec![self.skills_event(now)]
            }
            DataCommand::VerifyAudit(_) => vec![self.push(
                EventPayload::AuditStatus {
                    healthy: self.audit_healthy,
                    label: self.audit_detail.clone(),
                },
                None,
                now,
            )],
            DataCommand::LastTaskResult(_) => {
                let mut events = vec![self.push(
                    EventPayload::TaskHistory {
                        entries: self.history_mirror.clone(),
                    },
                    None,
                    now,
                )];
                let finished = self.orchestrator.last_finished().cloned();
                if let Some(record) = finished {
                    events.push(self.task_ended_event(&record, now));
                }
                events
            }
        }
    }

    fn submit_task(
        &mut self,
        text: &str,
        window: WindowId,
        tab: TabId,
        now: UnixMillis,
    ) -> Vec<OutboundEvent> {
        let Some(browser) = self.browser.clone() else {
            return self.emit_error(ErrorCase::BrowserHostUnavailable.copy(), None, now);
        };
        if browser.private {
            return self.emit_error(
                ErrorCase::PolicyBlocked.copy(),
                Some("The Agent is unavailable in Private Windows.".to_owned()),
                now,
            );
        }
        let context = SubmitContext {
            window,
            tab,
            origin: browser.origin.clone(),
            profile: ba_domain::ProfileKind::Normal,
        };
        match self.orchestrator.submit(text, context, now) {
            Ok(SubmitOutcome::Accepted { events, .. }) => self.wrap_task_events(events),
            Ok(SubmitOutcome::NeedsVaultInstead { detail }) => {
                self.emit_error(ErrorCase::PolicyBlocked.copy(), Some(detail), now)
            }
            Err(error) => self.emit_orchestrator_error(&error, now),
        }
    }

    fn orchestrator_event(
        &mut self,
        result: Result<Vec<ba_domain::TaskEvent>, OrchestratorError>,
        now: UnixMillis,
    ) -> Vec<OutboundEvent> {
        match result {
            Ok(events) => self.wrap_task_events(events),
            Err(error) => self.emit_orchestrator_error(&error, now),
        }
    }

    fn wrap_task_events(&mut self, events: Vec<ba_domain::TaskEvent>) -> Vec<OutboundEvent> {
        let now = events
            .last()
            .map_or(UnixMillis::from_millis(0), |event| event.at_unix_ms);
        events
            .into_iter()
            .map(|event| {
                self.push(
                    EventPayload::Task {
                        event: Box::new(event),
                    },
                    None,
                    now,
                )
            })
            .collect()
    }

    fn notes_event(&mut self, now: UnixMillis) -> OutboundEvent {
        let notes = self
            .notes
            .iter()
            .map(|note| NoteView {
                note: note.note.clone(),
                title: note.title.clone(),
                body: note.body.clone(),
                source_url: note.source_url.clone(),
                captured_at_unix_ms: note.captured_at_unix_ms,
            })
            .collect();
        self.push(EventPayload::Notes { notes }, None, now)
    }

    fn skills_event(&mut self, now: UnixMillis) -> OutboundEvent {
        let skills = self
            .skills
            .iter()
            .map(|skill| SkillView {
                skill: skill.skill.clone(),
                name: skill.name.clone(),
                version: skill.version.clone(),
                provenance: skill.provenance.clone(),
                enabled: skill.enabled,
            })
            .collect();
        self.push(EventPayload::Skills { skills }, None, now)
    }

    fn readiness_event(&mut self, now: UnixMillis) -> OutboundEvent {
        let progress = self.readiness.progress().map(|progress| {
            format!(
                "{}% ({} of {} bytes)",
                progress.percent(),
                progress.done_bytes,
                progress.total_bytes
            )
        });
        self.push(
            EventPayload::Readiness {
                state: self.readiness.state().code().to_owned(),
                label: self.readiness.label().to_owned(),
                controls: self
                    .readiness
                    .controls()
                    .iter()
                    .map(|control| (*control).to_owned())
                    .collect(),
                cause: self.readiness.cause().map(str::to_owned),
                progress,
            },
            None,
            now,
        )
    }

    fn task_ended_event(&mut self, record: &TaskRecord, now: UnixMillis) -> OutboundEvent {
        let verification_label = record
            .verification
            .map(ba_domain::verification_label)
            .map(str::to_owned);
        let detail = format!(
            "{} {}",
            record.summary_line(),
            verification_explanation(record.verification)
        );
        self.history_mirror.insert(
            0,
            TaskHistoryEntry {
                task_id: record.task_id.clone(),
                text: record.prefill_text(),
                state: record.state.code().to_owned(),
                label: ba_domain::task_state_label(record.state, None)
                    .unwrap_or("Task ended")
                    .to_owned(),
                verification: record.verification,
                verification_label,
                created_at_unix_ms: record.created_at_unix_ms,
                ended_at_unix_ms: record.ended_at_unix_ms,
            },
        );
        self.push(
            EventPayload::TaskEnded {
                task_id: record.task_id.clone(),
                state: record.state.code().to_owned(),
                state_label: ba_domain::task_state_label(record.state, None)
                    .unwrap_or("Task ended")
                    .to_owned(),
                verification: record.verification,
                verification_label: record
                    .verification
                    .map(ba_domain::verification_label)
                    .map(str::to_owned),
                detail,
            },
            Some(record.task_id.clone()),
            now,
        )
    }

    fn emit_orchestrator_error(
        &mut self,
        error: &OrchestratorError,
        now: UnixMillis,
    ) -> Vec<OutboundEvent> {
        let case = match error {
            OrchestratorError::TaskAlreadyRunning { .. } => None,
            OrchestratorError::PrivateWindowAgentDisabled => Some(ErrorCase::PolicyBlocked),
            OrchestratorError::ClarificationLimitReached => Some(ErrorCase::VerificationUnknown),
            _ => None,
        };
        match case {
            Some(case) => self.emit_error(case.copy(), Some(error.to_string()), now),
            None => {
                let payload = self.push(
                    EventPayload::CommandRejected {
                        command: "agent".to_owned(),
                        reason: error.to_string(),
                    },
                    self.orchestrator.active_task_id().cloned(),
                    now,
                );
                vec![payload]
            }
        }
    }

    fn emit_error(
        &mut self,
        copy: ba_domain::FourPartError,
        extra_known: Option<String>,
        now: UnixMillis,
    ) -> Vec<OutboundEvent> {
        let known = match extra_known {
            Some(extra) => format!("{} {}", copy.known, extra),
            None => copy.known.to_owned(),
        };
        vec![self.push(
            EventPayload::Error {
                error_id: copy.error_id.to_owned(),
                attempted: copy.attempted.to_owned(),
                observed: copy.observed.to_owned(),
                known,
                next_action: copy.next_action.to_owned(),
                rule: None,
            },
            self.orchestrator.active_task_id().cloned(),
            now,
        )]
    }

    /// Returns the approval view of the pending request, when one exists.
    #[must_use]
    pub fn pending_approval_view(&self) -> Option<ApprovalView> {
        let approval = self.orchestrator.pending_approval()?;
        Some(ApprovalView {
            action_id: approval.action_id.clone(),
            action: approval.presentation.action_kind.code().to_owned(),
            action_label: approval.presentation.action_label.clone(),
            origin: approval.presentation.origin.clone(),
            destination: approval.presentation.destination.clone(),
            target_label: approval.presentation.target_label.clone(),
            // The decision is bound to exactly what the user was shown, so the
            // digest, epoch and frame travel with the request.
            digest: approval.digest.to_string(),
            epoch: approval.epoch.to_string(),
            frame: approval.frame.to_string(),
            parameters: approval
                .presentation
                .parameters
                .iter()
                .map(|parameter| (parameter.name.clone(), parameter.value.clone()))
                .collect(),
            why: approval.presentation.why.clone(),
            rule: approval.rule.code().to_owned(),
            class: approval.presentation.class.code().to_owned(),
            expected_effect: approval.presentation.expected_effect.clone(),
            undo: approval.presentation.undo_description.clone(),
            expires_at_unix_ms: approval.expires_at_unix_ms,
        })
    }

    /// Returns the handoff view for a handed-to-user proposal.
    #[must_use]
    pub fn handoff_view(presentation: &ba_policy::HandoffPresentation) -> HandoffView {
        HandoffView {
            action: presentation.action_kind.code().to_owned(),
            action_label: presentation.action_label.clone(),
            origin: presentation.origin.clone(),
            destination: presentation.destination.clone(),
            target_label: presentation.target_label.clone(),
            parameters: presentation
                .parameters
                .iter()
                .map(|parameter| (parameter.name.clone(), parameter.value.clone()))
                .collect(),
            consequences: presentation.consequences.clone(),
            reason: presentation.reason.clone(),
            rule: presentation.rule.code().to_owned(),
            class: presentation.class.code().to_owned(),
            user_action_hint: presentation.user_action_hint.clone(),
        }
    }

    /// Returns the current status view for the active task.
    #[must_use]
    pub fn task_status_view(&self) -> Option<TaskStatusView> {
        let task_id = self.orchestrator.active_task_id()?.clone();
        let state = self.orchestrator.active_state()?;
        let budget = self.orchestrator.budget()?;
        let events = self.orchestrator.active_events();
        let last = events.last()?;
        Some(TaskStatusView {
            task_id,
            state: state.code().to_owned(),
            label: last.label.clone(),
            wait_reason: last.wait_reason.map(|reason| reason.code().to_owned()),
            action_class: last.action_class.map(|class| class.code().to_owned()),
            verification: last.verification,
            budget,
            elapsed_active_ms: budget.active_ms.used,
            elapsed_waiting_ms: budget.waiting_ms,
            last_event_at_unix_ms: last.at_unix_ms,
            task_tabs: self.orchestrator.task_tabs(),
        })
    }

    /// Returns the status view for the most recently finished task.
    #[must_use]
    pub fn last_task_status_view(&self) -> Option<TaskStatusView> {
        let record = self.orchestrator.last_finished()?;
        let last = record.events.last()?;
        Some(TaskStatusView {
            task_id: record.task_id.clone(),
            state: record.state.code().to_owned(),
            label: ba_domain::task_state_label(record.state, None)
                .unwrap_or("Task ended")
                .to_owned(),
            wait_reason: None,
            action_class: None,
            verification: record.verification,
            budget: last.budget.unwrap_or(ba_domain::TaskBudgetView {
                action_proposals: ba_domain::BudgetField { limit: 0, used: 0 },
                dispatched_actions: ba_domain::BudgetField { limit: 0, used: 0 },
                navigations: ba_domain::BudgetField { limit: 0, used: 0 },
                input_tokens: ba_domain::BudgetField { limit: 0, used: 0 },
                output_tokens: ba_domain::BudgetField { limit: 0, used: 0 },
                active_ms: ba_domain::BudgetField { limit: 0, used: 0 },
                waiting_ms: 0,
                extensions_used: 0,
            }),
            elapsed_active_ms: record
                .events
                .last()
                .map_or(0, |event| event.elapsed_active_ms),
            elapsed_waiting_ms: record
                .events
                .last()
                .map_or(0, |event| event.elapsed_waiting_ms),
            last_event_at_unix_ms: record.ended_at_unix_ms,
            task_tabs: Vec::new(),
        })
    }

    /// Returns the origin the core currently considers active.
    #[must_use]
    pub fn active_origin(&self) -> Option<&Origin> {
        self.browser.as_ref().map(|state| &state.origin)
    }

    /// Returns the downloads the host reported.
    #[must_use]
    pub fn downloads(&self) -> &[DownloadView] {
        &self.downloads
    }

    /// Reports downloads from the host.
    pub fn report_downloads(&mut self, downloads: Vec<DownloadView>) -> Vec<EventPayload> {
        self.downloads = downloads.clone();
        vec![EventPayload::Downloads { downloads }]
    }

    /// Reports measured model progress from the installer.
    pub fn report_model_progress(&mut self, progress: MeasuredProgress) -> Vec<EventPayload> {
        self.readiness.report_progress(progress);
        Vec::new()
    }

    /// Completes an installation with the downloaded bytes.
    ///
    /// # Errors
    ///
    /// Returns the integrity error; the readiness state becomes
    /// `INTEGRITY_FAILED` and no artifact is loaded.
    pub fn complete_model_install(
        &mut self,
        manifest: ModelManifest,
        bytes: &[u8],
    ) -> Result<(), ba_model::ModelError> {
        self.readiness.complete_install(manifest, bytes)
    }

    fn push(
        &mut self,
        payload: EventPayload,
        task_id: Option<TaskId>,
        _now: UnixMillis,
    ) -> OutboundEvent {
        let sequence = self.out_sequence;
        self.out_sequence = sequence.next().unwrap_or(sequence);
        OutboundEvent {
            sequence,
            payload,
            task_id,
        }
    }
}

fn verification_explanation(verification: Option<VerificationOutcome>) -> String {
    match verification {
        Some(outcome) => ba_domain::verification_meaning(outcome).to_owned(),
        None => "No result check was applicable for the step that ended the task.".to_owned(),
    }
}

/// Trusted host facts used to build a policy context.
#[derive(Debug, Clone)]
pub struct HostFacts {
    /// Browser context identity.
    pub profile: ProfileIdentity,
    /// Origins already opened or approved for this task.
    pub approved_origins: Vec<Origin>,
    /// Whether `Only use sites I've already opened` is enabled.
    pub origin_allowlist_enabled: bool,
    /// Whether durable audit persistence is healthy.
    pub audit_healthy: bool,
    /// Whether the platform credential store is available.
    pub vault_available: bool,
    /// Whether credential use was enabled for this exact origin.
    pub secure_fill_authorized: bool,
}

/// Builds the policy context for the active tab from trusted core state.
///
/// The tab, frame, origin and epoch come from the core's own browser state, so a
/// UI payload can never substitute a different origin.
#[must_use]
pub fn policy_context(
    task_id: TaskId,
    browser: &BrowserStateView,
    epoch: &ObservationId,
    facts: &HostFacts,
) -> PolicyContext {
    PolicyContext {
        task_id,
        tab: browser.tab.clone(),
        frame: fallback_frame(),
        origin: browser.origin.clone(),
        epoch: epoch.clone(),
        profile: facts.profile.clone(),
        approved_origins: facts.approved_origins.clone(),
        origin_allowlist_enabled: facts.origin_allowlist_enabled,
        audit_healthy: facts.audit_healthy,
        vault_available: facts.vault_available,
        secure_fill_authorized: facts.secure_fill_authorized,
    }
}

fn fallback_frame() -> FrameId {
    FrameId::from_number("frame", 0)
}

/// Evaluates a proposal against trusted policy through the orchestrator.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the orchestrator cannot evaluate the request.
pub fn propose(
    core: &mut AppCore,
    request: &PolicyRequest<'_>,
    now: UnixMillis,
) -> Result<Vec<OutboundEvent>, RuntimeError> {
    let outcome = core.orchestrator.propose(request, now)?;
    let mut payloads = core.wrap_task_events(outcome.events);
    match outcome.disposition {
        ProposalDisposition::Authorized { .. } => {}
        ProposalDisposition::AwaitingApproval { .. } => {
            if let Some(view) = core.pending_approval_view() {
                payloads.push(core.push(
                    EventPayload::ApprovalRequested {
                        approval: Box::new(view),
                    },
                    core.orchestrator.active_task_id().cloned(),
                    now,
                ));
            }
        }
        ProposalDisposition::HandedToUser { presentation } => {
            payloads.push(core.push(
                EventPayload::HandoffRequired {
                    handoff: Box::new(AppCore::handoff_view(&presentation)),
                },
                core.orchestrator.active_task_id().cloned(),
                now,
            ));
        }
        ProposalDisposition::Refused { rule, reason } => {
            let rule_code = rule.code().to_owned();
            payloads.push(core.push(
                EventPayload::Error {
                    error_id: ErrorCase::PolicyBlocked.copy().error_id.to_owned(),
                    attempted: ErrorCase::PolicyBlocked.copy().attempted.to_owned(),
                    observed: ErrorCase::PolicyBlocked.copy().observed.to_owned(),
                    known: reason,
                    next_action: ErrorCase::PolicyBlocked.copy().next_action.to_owned(),
                    rule: Some(rule_code),
                },
                core.orchestrator.active_task_id().cloned(),
                now,
            ));
        }
        ProposalDisposition::BudgetReached => {}
    }
    Ok(payloads)
}

/// Validates an inbound frame and reports the rejection as an event.
///
/// # Errors
///
/// Returns the [`ba_ipc::IpcError`] so the caller can write the rejection event.
pub fn decode_or_reject(
    frame: &[u8],
    tracker: &mut SequenceTracker,
) -> Result<Command, ba_ipc::IpcError> {
    ba_ipc::decode_command(frame, tracker).map(|envelope| envelope.payload)
}

/// Returns the blocking rule for a refusal, when the event carries one.
#[must_use]
pub fn blocking_rule(event: &EventPayload) -> Option<RuleId> {
    match event {
        EventPayload::Error {
            rule: Some(rule), ..
        } => rule.split('.').next().and_then(|_| rule_from_code(rule)),
        _ => None,
    }
}

/// Maps a rule code back to its identifier, used by the diagnostic view.
#[must_use]
pub fn rule_from_code(code: &str) -> Option<RuleId> {
    const ALL: [RuleId; 22] = [
        RuleId::AgentDisabledInPrivateWindow,
        RuleId::ClassifiedRead,
        RuleId::ClassifiedReversibleWrite,
        RuleId::ClassifiedIrreversible,
        RuleId::ClassifiedUnknown,
        RuleId::CredentialsInUrl,
        RuleId::OriginOutsideAllowlist,
        RuleId::AuditUnavailable,
        RuleId::VaultUnavailable,
        RuleId::WeakReversibilityEvidence,
        RuleId::CapabilityRecordExpired,
        RuleId::CapabilityRecordMismatch,
        RuleId::SecureFillNotAuthorized,
        RuleId::SecureFillRequiresApproval,
        RuleId::ReversibleWriteNeedsApproval,
        RuleId::IrreversibleRequiresHandoff,
        RuleId::UnknownRequiresHandoff,
        RuleId::EgressNotRequested,
        RuleId::CredentialOriginMismatch,
        RuleId::RawSecretEgress,
        RuleId::PageSuppliedEgress,
        RuleId::Allowed,
    ];
    ALL.into_iter().find(|rule| rule.code() == code)
}

/// Serializes an outbound event for the transport.
///
/// # Errors
///
/// Returns [`ba_ipc::IpcError::Encode`] when the envelope cannot be serialized.
pub fn encode_outbound(
    event: &OutboundEvent,
    at_unix_ms: UnixMillis,
    monotonic_ms: u64,
) -> Result<String, ba_ipc::IpcError> {
    let envelope = ba_ipc::EventEnvelope::new(
        event.sequence,
        at_unix_ms,
        monotonic_ms,
        event.task_id.clone(),
        None,
        event.payload.clone(),
    );
    ba_ipc::encode_event(&envelope)
}

/// Renders any serializable value as a single JSON line.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the value cannot be serialized.
pub fn to_jsonl<T: Serialize>(value: &T) -> Result<String, RuntimeError> {
    let mut line = serde_json::to_string(value).map_err(|_| RuntimeError::HostUnavailable)?;
    line.push('\n');
    Ok(line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ba_domain::{BrowserContextId, ProfileKind};

    fn now() -> UnixMillis {
        UnixMillis::from_millis(1_700_000_000_000)
    }

    fn browser_state() -> BrowserStateView {
        BrowserStateView {
            window: WindowId::parse("window-1").unwrap(),
            tab: TabId::parse("tab-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            context: BrowserContextId::parse("ctx-normal").unwrap(),
            private: false,
            address: "https://example.com/".to_owned(),
            security_label: "Secure".to_owned(),
            title: "Example".to_owned(),
            loading_state: "Loaded".to_owned(),
            can_go_back: false,
            can_go_forward: false,
        }
    }

    fn command(payload: &str) -> Command {
        let frame = format!(
            r#"{{"schema_version":{},"sequence":1,"at_unix_ms":1700000000000,"payload":{payload}}}"#,
            ba_domain::SCHEMA_VERSION
        );
        let mut tracker = SequenceTracker::new();
        decode_or_reject(frame.as_bytes(), &mut tracker).unwrap()
    }

    #[test]
    fn browser_commands_report_the_missing_host_without_simulating_a_page() {
        let mut core = AppCore::new();
        let events = core.handle(
            &command(r#"{"domain":"browser","command":{"op":"NAVIGATE","args":{"url":"https://example.com"}}}"#),
            now(),
        );
        assert_eq!(events.len(), 1);
        match &events[0].payload {
            EventPayload::Error {
                error_id,
                known,
                next_action,
                ..
            } => {
                assert_eq!(error_id, "BA-ERR-BROWSER-001");
                assert!(known.contains("Nothing was simulated"));
                assert!(next_action.contains("managed browser host"));
            }
            other => panic!("expected an error event, got {other:?}"),
        }
        assert!(core.active_origin().is_none());
    }

    #[test]
    fn a_task_cannot_be_submitted_without_the_managed_host() {
        let mut core = AppCore::new();
        let events = core.handle(
            &command(
                r#"{"domain":"agent","command":{"op":"SUBMIT_TASK","args":{"text":"open the pricing page","window":"window-1","tab":"tab-1"}}}"#,
            ),
            now(),
        );
        assert!(matches!(events[0].payload, EventPayload::Error { .. }));
        assert!(core.orchestrator().active_task_id().is_none());
    }

    #[test]
    fn a_task_submitted_with_the_host_attached_emits_canonical_copy() {
        let mut core = AppCore::new();
        core.report_browser_state(Some(browser_state()));
        let events = core.handle(
            &command(
                r#"{"domain":"agent","command":{"op":"SUBMIT_TASK","args":{"text":"open the pricing page","window":"window-1","tab":"tab-1"}}}"#,
            ),
            now(),
        );
        match &events[0].payload {
            EventPayload::Task { event } => {
                assert_eq!(event.state.code(), "CREATED");
                assert_eq!(event.label, "Task received");
            }
            other => panic!("expected a task event, got {other:?}"),
        }
        assert_eq!(
            core.active_origin().map(Origin::canonical),
            Some("https://example.com".to_owned())
        );
    }

    #[test]
    fn a_second_task_is_rejected_without_a_hidden_queue() {
        let mut core = AppCore::new();
        core.report_browser_state(Some(browser_state()));
        let submit = command(
            r#"{"domain":"agent","command":{"op":"SUBMIT_TASK","args":{"text":"open the pricing page","window":"window-1","tab":"tab-1"}}}"#,
        );
        core.handle(&submit, now());
        let events = core.handle(&submit, now());
        match &events[0].payload {
            EventPayload::CommandRejected { reason, .. } => {
                assert!(reason.contains("already running"));
            }
            other => panic!("expected a rejection, got {other:?}"),
        }
    }

    #[test]
    fn private_browser_state_blocks_submission() {
        let mut core = AppCore::new();
        let mut state = browser_state();
        state.private = true;
        core.report_browser_state(Some(state));
        let events = core.handle(
            &command(
                r#"{"domain":"agent","command":{"op":"SUBMIT_TASK","args":{"text":"open the pricing page","window":"window-1","tab":"tab-1"}}}"#,
            ),
            now(),
        );
        match &events[0].payload {
            EventPayload::Error { known, .. } => {
                assert!(known.contains("unavailable in Private Windows"));
            }
            other => panic!("expected an error, got {other:?}"),
        }
    }

    /// The approval the UI renders must be bound to exactly what it showed:
    /// the digest, epoch and frame travel with the request so the decision can
    /// be checked against the current observation ([BA-POLICY-002]).
    #[test]
    fn an_approval_request_carries_the_binding_the_user_was_shown() {
        use ba_domain::{Action, BoundedText, NavigateParams, ObservationId, ValidatedAction};
        use ba_policy::{DataEgress, EgressProvenance, PolicyRequest, ProfileIdentity};

        let mut core = AppCore::new();
        core.report_browser_state(Some(browser_state()));
        core.handle(
            &command(
                r#"{"domain":"agent","command":{"op":"SUBMIT_TASK","args":{"text":"open the pricing page","window":"window-1","tab":"tab-1"}}}"#,
            ),
            now(),
        );
        let task_id = core
            .orchestrator()
            .active_task_id()
            .cloned()
            .expect("the task was accepted");
        core.begin_observation(now()).unwrap();
        let epoch = ObservationId::parse("epoch-1").unwrap();
        let facts = HostFacts {
            profile: ProfileIdentity {
                context: BrowserContextId::parse("ctx-normal").unwrap(),
                kind: ProfileKind::Normal,
            },
            approved_origins: Vec::new(),
            origin_allowlist_enabled: false,
            audit_healthy: true,
            vault_available: true,
            secure_fill_authorized: false,
        };
        let action = ValidatedAction::new(Action::Navigate(NavigateParams {
            url: BoundedText::parse("https://example.com/checkout", "navigate.url", 2_048).unwrap(),
        }))
        .unwrap();
        let request = PolicyRequest {
            context: policy_context(task_id, &browser_state(), &epoch, &facts),
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

        let events = propose(&mut core, &request, now()).unwrap();
        let approval = events
            .iter()
            .find_map(|event| match &event.payload {
                EventPayload::ApprovalRequested { approval } => Some(approval.clone()),
                _ => None,
            })
            .expect("an approvable write must produce an approval request");

        assert_eq!(approval.digest, action.digest().to_string());
        assert_eq!(approval.epoch, "epoch-1");
        assert_eq!(approval.frame, fallback_frame().to_string());
        assert_eq!(approval.origin, "https://example.com");
    }

    #[test]
    fn model_status_is_truthful_about_an_uninstalled_model() {
        let mut core = AppCore::new();
        let events = core.handle(
            &command(r#"{"domain":"model","command":{"op":"STATUS","args":{}}}"#),
            now(),
        );
        match &events[0].payload {
            EventPayload::Readiness {
                state,
                label,
                controls,
                ..
            } => {
                assert_eq!(state, "NOT_INSTALLED");
                assert_eq!(label, "Model not installed");
                assert_eq!(controls, &vec!["Install model".to_owned()]);
            }
            other => panic!("expected readiness, got {other:?}"),
        }
        assert!(!core.readiness_state().accepts_tasks());
    }

    #[test]
    fn installing_an_unregistered_artifact_downloads_nothing() {
        let mut core = AppCore::new();
        let events = core.handle(
            &command(
                r#"{"domain":"model","command":{"op":"INSTALL","args":{"manifest_id":"missing"}}}"#,
            ),
            now(),
        );
        match &events[0].payload {
            EventPayload::Error { known, .. } => {
                assert!(known.contains("nothing was downloaded"));
            }
            other => panic!("expected an error, got {other:?}"),
        }
        assert_eq!(core.readiness_state(), ReadinessState::NotInstalled);
    }

    #[test]
    fn notes_and_skills_are_local_user_actions() {
        let mut core = AppCore::new();
        let notes = core.handle(
            &command(
                r#"{"domain":"data","command":{"op":"CREATE_NOTE","args":{"title":"Fare rules","body":"Peak hours cost more.","source_url":"https://example.com/fares"}}}"#,
            ),
            now(),
        );
        match &notes[0].payload {
            EventPayload::Notes { notes } => {
                assert_eq!(notes.len(), 1);
                assert_eq!(notes[0].title, "Fare rules");
                assert_eq!(notes[0].captured_at_unix_ms, now());
            }
            other => panic!("expected notes, got {other:?}"),
        }

        core.register_skill(Skill {
            skill: ba_domain::SkillId::parse("skill-1").unwrap(),
            name: "Open the fare table".to_owned(),
            version: "1".to_owned(),
            provenance: "saved from task-1 and reviewed".to_owned(),
            enabled: true,
        });
        let listed = core.handle(
            &command(r#"{"domain":"data","command":{"op":"LIST_SKILLS","args":{}}}"#),
            now(),
        );
        match &listed[0].payload {
            EventPayload::Skills { skills } => {
                assert_eq!(skills.len(), 1);
                assert!(skills[0].enabled);
            }
            other => panic!("expected skills, got {other:?}"),
        }
        let revoked = core.handle(
            &command(
                r#"{"domain":"data","command":{"op":"REVOKE_SKILL","args":{"skill":"skill-1"}}}"#,
            ),
            now(),
        );
        match &revoked[0].payload {
            EventPayload::Skills { skills } => assert!(!skills[0].enabled),
            other => panic!("expected skills, got {other:?}"),
        }
    }

    #[test]
    fn audit_status_never_claims_a_verified_chain_when_degraded() {
        let mut core = AppCore::new();
        core.set_audit_health(
            false,
            "Integrity check failed in segment segment-1 at record 4",
        );
        let events = core.handle(
            &command(r#"{"domain":"data","command":{"op":"VERIFY_AUDIT","args":{}}}"#),
            now(),
        );
        match &events[0].payload {
            EventPayload::AuditStatus { healthy, label } => {
                assert!(!healthy);
                assert!(!label.starts_with("Integrity verified"));
            }
            other => panic!("expected audit status, got {other:?}"),
        }
    }

    #[test]
    fn policy_context_uses_the_cores_own_browser_state() {
        let state = browser_state();
        let profile = ProfileIdentity {
            context: state.context.clone(),
            kind: ProfileKind::Normal,
        };
        let epoch = ObservationId::parse("epoch-1").unwrap();
        let facts = HostFacts {
            profile,
            approved_origins: vec![Origin::parse("https://example.com").unwrap()],
            origin_allowlist_enabled: true,
            audit_healthy: true,
            vault_available: false,
            secure_fill_authorized: false,
        };
        let context = policy_context(TaskId::parse("task-1").unwrap(), &state, &epoch, &facts);
        assert_eq!(context.origin.canonical(), "https://example.com");
        assert!(context.origin_allowlist_enabled);
    }

    #[test]
    fn events_are_sequenced_and_encodable() {
        let mut core = AppCore::new();
        let events = core.handle(
            &command(r#"{"domain":"model","command":{"op":"STATUS","args":{}}}"#),
            now(),
        );
        assert_eq!(events[0].sequence, Sequence::new(1));
        let line = encode_outbound(&events[0], now(), 10).unwrap();
        assert!(line.ends_with('\n'));
        assert!(line.contains("\"kind\":\"READINESS\""));
    }

    #[test]
    fn a_finished_task_produces_a_history_entry() {
        let mut core = AppCore::new();
        core.report_browser_state(Some(browser_state()));
        let submit = command(
            r#"{"domain":"agent","command":{"op":"SUBMIT_TASK","args":{"text":"open the pricing page","window":"window-1","tab":"tab-1"}}}"#,
        );
        core.handle(&submit, now());
        core.handle(
            &command(r#"{"domain":"agent","command":{"op":"CANCEL_TASK","args":{}}}"#),
            now(),
        );
        let events = core.handle(
            &command(r#"{"domain":"data","command":{"op":"LAST_TASK_RESULT","args":{}}}"#),
            now(),
        );
        let ended = events
            .iter()
            .find_map(|event| match &event.payload {
                EventPayload::TaskEnded {
                    state,
                    verification,
                    detail,
                    ..
                } => Some((state.clone(), *verification, detail.clone())),
                _ => None,
            })
            .expect("the finished task is reported");
        assert_eq!(ended.0, "CANCELLED");
        assert!(ended.1.is_none());
        assert!(ended.2.contains("Verification not run"));
    }

    #[test]
    fn rule_codes_round_trip_for_the_diagnostic_view() {
        assert_eq!(
            rule_from_code("BA-EGRESS-001.credential-origin-mismatch"),
            Some(RuleId::CredentialOriginMismatch)
        );
        assert_eq!(rule_from_code("not-a-rule"), None);
    }
}
