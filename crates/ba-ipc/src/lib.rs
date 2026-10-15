//! Browser-Agent IPC contract (Section 20.2).
//!
//! The first-party UI is an untrusted presentation client even though its
//! source is first-party. Every inbound frame is size-capped before it is
//! deserialized, schema-versioned, sequence-checked and validated against a
//! closed schema. Unknown variants, unknown fields, duplicate or stale sequence
//! values, oversized payloads and schema-version mismatches are rejected and
//! never grant authority.
//!
//! The reverse direction is a closed set of typed events, each carrying a
//! schema version, a monotonic sequence and, where applicable, task identity.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![deny(missing_docs)]

use ba_domain::{
    BrowserContextId, FrameId, MAX_IPC_FRAME_BYTES, ObservationId, Origin, ReadinessState,
    SCHEMA_VERSION, Sequence, TabId, TaskEvent, TaskId, UnixMillis, VerificationOutcome, WindowId,
};
use serde::{Deserialize, Serialize};

/// An operation that carries no arguments.
///
/// Operations without arguments still carry a closed, empty argument object so
/// that an unexpected field is a schema error rather than silently ignored data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoArgs {}

/// Browser navigation and tab operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BrowserCommand {
    /// Navigate the active tab to a URL.
    Navigate(NavigateArgs),
    /// Go back in the active tab.
    Back(NoArgs),
    /// Go forward in the active tab.
    Forward(NoArgs),
    /// Reload the active tab.
    Reload(NoArgs),
    /// Stop loading the active tab.
    Stop(NoArgs),
    /// Open a new tab.
    NewTab(NewTabArgs),
    /// Close a tab.
    CloseTab(TabArgs),
    /// Activate a tab.
    ActivateTab(TabArgs),
    /// Find in page.
    FindInPage(FindArgs),
    /// Focus the address field.
    FocusAddressField(NoArgs),
}

/// Arguments for [`BrowserCommand::Navigate`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigateArgs {
    /// Destination URL.
    pub url: String,
}

/// Arguments for [`BrowserCommand::NewTab`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewTabArgs {
    /// Initial URL; the local new-tab page when empty.
    pub url: Option<String>,
}

/// Arguments naming a tab.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TabArgs {
    /// Tab identity.
    pub tab: TabId,
}

/// Arguments for [`BrowserCommand::FindInPage`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindArgs {
    /// Query text.
    pub query: String,
}

/// Window lifecycle operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WindowCommand {
    /// Open a normal window.
    New(NoArgs),
    /// Open a private window.
    NewPrivate(NoArgs),
    /// Close a window.
    Close(WindowArgs),
}

/// Arguments naming a window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowArgs {
    /// Window identity.
    pub window: WindowId,
}

/// Agent operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentCommand {
    /// Submit a task instruction.
    SubmitTask(SubmitTaskArgs),
    /// Pause the active task.
    Pause(NoArgs),
    /// Resume after an explicit stop.
    Resume(NoArgs),
    /// Cancel the active task.
    CancelTask(NoArgs),
    /// Hand the browser to the user.
    TakeOver(NoArgs),
    /// Approve the pending bounded action.
    ApproveAction(ApproveArgs),
    /// Cancel the pending approval.
    CancelApproval(NoArgs),
    /// Extend the exhausted budget by up to 50%, once.
    ExtendBudget(NoArgs),
    /// Finish with the results gathered so far.
    FinishWithResults(NoArgs),
}

/// Arguments for [`AgentCommand::SubmitTask`].
///
/// The tab's origin is deliberately absent: the runtime takes the origin from
/// its own browser state rather than trusting the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitTaskArgs {
    /// Instruction text.
    pub text: String,
    /// Window the task was submitted from.
    pub window: WindowId,
    /// Tab the task was submitted from.
    pub tab: TabId,
}

/// Arguments for [`AgentCommand::ApproveAction`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApproveArgs {
    /// Action the user approved.
    pub action_id: ba_domain::ActionId,
    /// Digest of the action as the UI was shown it.
    pub digest: ba_domain::ActionDigest,
    /// Origin shown in the request.
    pub origin: Origin,
    /// Observation epoch shown in the request.
    pub epoch: ObservationId,
    /// Frame shown in the request.
    pub frame: FrameId,
}

/// Model lifecycle operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelCommand {
    /// Ask for the current readiness state.
    Status(NoArgs),
    /// Start an explicit, cancellable installation.
    Install(InstallArgs),
    /// Cancel an installation.
    CancelInstall(NoArgs),
    /// Re-hash the installed artifact.
    Verify(NoArgs),
    /// Resume a parked model.
    Resume(NoArgs),
}

/// Arguments for [`ModelCommand::Install`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallArgs {
    /// Manifest identifier to install.
    pub manifest_id: String,
}

/// Local data operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataCommand {
    /// Remove one origin's normal-profile data.
    ForgetSite(ForgetSiteArgs),
    /// Clear browsing data with a category selection.
    ClearBrowsingData(NoArgs),
    /// Clear Agent task history.
    ClearTaskHistory(NoArgs),
    /// List local research notes.
    ListNotes(NoArgs),
    /// Create a local research note.
    CreateNote(CreateNoteArgs),
    /// Delete a local research note.
    DeleteNote(NoteArgs),
    /// List reviewed skills.
    ListSkills(NoArgs),
    /// Disable or revoke a skill.
    RevokeSkill(SkillArgs),
    /// Verify the retained audit chain.
    VerifyAudit(NoArgs),
    /// Report the stored result of the last task.
    LastTaskResult(NoArgs),
}

/// Arguments for [`DataCommand::ForgetSite`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForgetSiteArgs {
    /// Exact origin to forget.
    pub origin: Origin,
}

/// Arguments for [`DataCommand::CreateNote`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateNoteArgs {
    /// Note title.
    pub title: String,
    /// Note body.
    pub body: String,
    /// Source URL the excerpt came from.
    pub source_url: String,
}

/// Arguments naming a note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoteArgs {
    /// Note identity.
    pub note: ba_domain::NoteId,
}

/// Arguments naming a skill.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillArgs {
    /// Skill identity.
    pub skill: ba_domain::SkillId,
}

/// A command from the first-party UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "domain", content = "command", rename_all = "snake_case")]
pub enum Command {
    /// Browser navigation and tab operations.
    Browser(BrowserCommand),
    /// Window lifecycle operations.
    Window(WindowCommand),
    /// Agent operations.
    Agent(AgentCommand),
    /// Model lifecycle operations.
    Model(ModelCommand),
    /// Local data operations.
    Data(DataCommand),
}

impl Command {
    /// Returns the stable command name used in diagnostics and audit records.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Browser(command) => match command {
                BrowserCommand::Navigate(_) => "browser.navigate",
                BrowserCommand::Back(_) => "browser.back",
                BrowserCommand::Forward(_) => "browser.forward",
                BrowserCommand::Reload(_) => "browser.reload",
                BrowserCommand::Stop(_) => "browser.stop",
                BrowserCommand::NewTab(_) => "browser.newTab",
                BrowserCommand::CloseTab(_) => "browser.closeTab",
                BrowserCommand::ActivateTab(_) => "browser.activateTab",
                BrowserCommand::FindInPage(_) => "browser.findInPage",
                BrowserCommand::FocusAddressField(_) => "browser.focusAddressField",
            },
            Self::Window(command) => match command {
                WindowCommand::New(_) => "window.new",
                WindowCommand::NewPrivate(_) => "window.newPrivate",
                WindowCommand::Close(_) => "window.close",
            },
            Self::Agent(command) => match command {
                AgentCommand::SubmitTask(_) => "agent.submitTask",
                AgentCommand::Pause(_) => "agent.pause",
                AgentCommand::Resume(_) => "agent.resume",
                AgentCommand::CancelTask(_) => "agent.cancelTask",
                AgentCommand::TakeOver(_) => "agent.takeOver",
                AgentCommand::ApproveAction(_) => "agent.approveAction",
                AgentCommand::CancelApproval(_) => "agent.cancelApproval",
                AgentCommand::ExtendBudget(_) => "agent.extendBudget",
                AgentCommand::FinishWithResults(_) => "agent.finishWithResults",
            },
            Self::Model(command) => match command {
                ModelCommand::Status(_) => "model.status",
                ModelCommand::Install(_) => "model.install",
                ModelCommand::CancelInstall(_) => "model.cancelInstall",
                ModelCommand::Verify(_) => "model.verify",
                ModelCommand::Resume(_) => "model.resume",
            },
            Self::Data(command) => match command {
                DataCommand::ForgetSite(_) => "data.forgetSite",
                DataCommand::ClearBrowsingData(_) => "data.clearBrowsingData",
                DataCommand::ClearTaskHistory(_) => "data.clearTaskHistory",
                DataCommand::ListNotes(_) => "data.listNotes",
                DataCommand::CreateNote(_) => "data.createNote",
                DataCommand::DeleteNote(_) => "data.deleteNote",
                DataCommand::ListSkills(_) => "data.listSkills",
                DataCommand::RevokeSkill(_) => "data.revokeSkill",
                DataCommand::VerifyAudit(_) => "data.verifyAudit",
                DataCommand::LastTaskResult(_) => "data.lastTaskResult",
            },
        }
    }
}

/// A versioned inbound command envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandEnvelope {
    /// Wire schema version; must equal [`ba_domain::SCHEMA_VERSION`].
    pub schema_version: u16,
    /// Monotonic client sequence.
    pub sequence: Sequence,
    /// Client wall-clock time, used only for diagnostic correlation.
    pub at_unix_ms: UnixMillis,
    /// The command.
    pub payload: Command,
}

/// One entry of the local Agent task history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskHistoryEntry {
    /// Task identity.
    pub task_id: TaskId,
    /// Masked instruction text.
    pub text: String,
    /// Terminal state code.
    pub state: String,
    /// Terminal state label from the canonical table.
    pub label: String,
    /// Verification outcome, when one applies.
    pub verification: Option<VerificationOutcome>,
    /// Verification label, when one applies.
    pub verification_label: Option<String>,
    /// Creation time.
    pub created_at_unix_ms: UnixMillis,
    /// Terminal time.
    pub ended_at_unix_ms: UnixMillis,
}

/// One structured approval request shown to the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalView {
    /// Action the user is asked about.
    pub action_id: ba_domain::ActionId,
    /// Action verb.
    pub action: String,
    /// Plain-language verb label.
    pub action_label: String,
    /// Exact origin.
    pub origin: String,
    /// Destination, when the action targets one.
    pub destination: Option<String>,
    /// Target description.
    pub target_label: Option<String>,
    /// Digest of the exact action the user is shown.
    ///
    /// The approval decision must echo this value: an approval is bound to what
    /// was displayed, not to whatever action is current when the reply arrives.
    pub digest: String,
    /// Observation epoch shown in the request.
    pub epoch: String,
    /// Frame shown in the request.
    pub frame: String,
    /// Non-secret parameters, in display order.
    pub parameters: Vec<(String, String)>,
    /// Why approval is required.
    pub why: String,
    /// Rule code, for diagnostic detail.
    pub rule: String,
    /// Trusted consequence class code.
    pub class: String,
    /// Expected effect.
    pub expected_effect: String,
    /// What Undo will attempt, when a tested compensation exists.
    pub undo: Option<String>,
    /// When the request stops being actionable.
    pub expires_at_unix_ms: UnixMillis,
}

/// One structured handoff prompt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffView {
    /// Action verb.
    pub action: String,
    /// Plain-language verb label.
    pub action_label: String,
    /// Exact origin.
    pub origin: String,
    /// Destination, when the action targets one.
    pub destination: Option<String>,
    /// Target description.
    pub target_label: Option<String>,
    /// Non-secret parameters.
    pub parameters: Vec<(String, String)>,
    /// Known consequences.
    pub consequences: String,
    /// Why the agent stopped.
    pub reason: String,
    /// Rule code.
    pub rule: String,
    /// Trusted consequence class code.
    pub class: String,
    /// What the user should do next.
    pub user_action_hint: String,
}

/// The budget and elapsed-time figures shown while a task is active.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskStatusView {
    /// Owning task.
    pub task_id: TaskId,
    /// Lifecycle state code.
    pub state: String,
    /// Canonical lifecycle label.
    pub label: String,
    /// Wait reason code, when applicable.
    pub wait_reason: Option<String>,
    /// Trusted consequence class of the action under evaluation.
    pub action_class: Option<String>,
    /// Verification outcome, when one applies.
    pub verification: Option<VerificationOutcome>,
    /// Budget snapshot.
    pub budget: ba_domain::TaskBudgetView,
    /// Active execution time, in milliseconds.
    pub elapsed_active_ms: u64,
    /// Waiting and paused wall time, in milliseconds.
    pub elapsed_waiting_ms: u64,
    /// Time of the most recent runtime event.
    pub last_event_at_unix_ms: UnixMillis,
    /// Tabs in the task scope.
    pub task_tabs: Vec<TabId>,
}

/// One local research note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoteView {
    /// Note identity.
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

/// One reviewed skill.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillView {
    /// Skill identity.
    pub skill: ba_domain::SkillId,
    /// Display name.
    pub name: String,
    /// Immutable version label.
    pub version: String,
    /// Provenance description.
    pub provenance: String,
    /// Whether the skill is currently enabled.
    pub enabled: bool,
}

/// One download shown in the Downloads view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DownloadView {
    /// File name.
    pub file_name: String,
    /// Initiating origin.
    pub origin: String,
    /// Provenance label, for example `Requested by Agent task task-3`.
    pub provenance: String,
    /// Textual progress, for example `62%`.
    pub progress: String,
}

/// Browser state reported by the managed host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserStateView {
    /// Active window.
    pub window: WindowId,
    /// Active tab.
    pub tab: TabId,
    /// Exact origin of the active tab.
    pub origin: Origin,
    /// Request context identity.
    pub context: BrowserContextId,
    /// Whether this is a private window.
    pub private: bool,
    /// Canonical address text.
    pub address: String,
    /// Transport label for the address bar.
    pub security_label: String,
    /// Page title.
    pub title: String,
    /// Textual loading state.
    pub loading_state: String,
    /// Whether the active tab can go back in its own navigation history.
    ///
    /// The host owns this fact: the interface may not infer navigation
    /// availability from anything else it happens to hold (Section 2.5).
    pub can_go_back: bool,
    /// Whether the active tab can go forward in its own navigation history.
    pub can_go_forward: bool,
}

/// Events from the runtime to the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventPayload {
    /// An authoritative task lifecycle event.
    Task {
        /// The event.
        event: Box<TaskEvent>,
    },
    /// Current task status, budget and elapsed time.
    TaskStatus {
        /// The status.
        status: Box<TaskStatusView>,
    },
    /// The active task ended.
    TaskEnded {
        /// Task identity.
        task_id: TaskId,
        /// Terminal state code.
        state: String,
        /// Terminal state label.
        state_label: String,
        /// Verification outcome, when one applies.
        verification: Option<VerificationOutcome>,
        /// Verification label, when one applies.
        verification_label: Option<String>,
        /// Plain-language explanation of what is known.
        detail: String,
    },
    /// Browser state changed.
    BrowserState {
        /// Current state, or `None` when no managed host is attached.
        state: Option<Box<BrowserStateView>>,
    },
    /// The set of open tabs changed.
    Tabs {
        /// Open tabs with their titles.
        tabs: Vec<(TabId, String)>,
        /// Active tab.
        active: Option<TabId>,
    },
    /// Model readiness changed.
    Readiness {
        /// Readiness state code.
        state: String,
        /// Canonical label.
        label: String,
        /// Controls the state actually supports.
        controls: Vec<String>,
        /// Measured cause, when known.
        cause: Option<String>,
        /// Textual progress, when measurable.
        progress: Option<String>,
    },
    /// A structured approval request is pending.
    ApprovalRequested {
        /// The request.
        approval: Box<ApprovalView>,
    },
    /// A pending approval left the book.
    ApprovalCleared {
        /// How it ended.
        cause: String,
        /// Plain-language explanation.
        detail: String,
    },
    /// The user must perform this step.
    HandoffRequired {
        /// The prompt.
        handoff: Box<HandoffView>,
    },
    /// Local task history changed.
    TaskHistory {
        /// Entries, newest first.
        entries: Vec<TaskHistoryEntry>,
    },
    /// Local research notes changed.
    Notes {
        /// Notes.
        notes: Vec<NoteView>,
    },
    /// Skills changed.
    Skills {
        /// Skills.
        skills: Vec<SkillView>,
    },
    /// Downloads changed.
    Downloads {
        /// Downloads.
        downloads: Vec<DownloadView>,
    },
    /// Audit integrity status.
    AuditStatus {
        /// Whether the log is healthy.
        healthy: bool,
        /// User-facing status sentence, which never claims a verified chain when
        /// it is broken.
        label: String,
    },
    /// A user-first error with the four-part contract.
    Error {
        /// Error identifier, shown only in diagnostic detail.
        error_id: String,
        /// What was attempted.
        attempted: String,
        /// What was observed.
        observed: String,
        /// What is known or unknown now.
        known: String,
        /// Safe next action.
        next_action: String,
        /// Blocking rule code, when the error came from policy.
        rule: Option<String>,
    },
    /// A command was rejected.
    CommandRejected {
        /// Command name.
        command: String,
        /// Why it was rejected.
        reason: String,
    },
}

/// A versioned outbound event envelope.
pub type EventEnvelope = ba_domain::Envelope<EventPayload>;

/// Errors raised while decoding an inbound frame.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IpcError {
    /// The frame exceeded the hard cap.
    #[error("frame of {size} bytes exceeds the {limit} byte limit")]
    FrameTooLarge {
        /// Measured size.
        size: usize,
        /// Hard cap.
        limit: usize,
    },
    /// The frame was empty.
    #[error("frame is empty")]
    EmptyFrame,
    /// The frame was not valid JSON.
    #[error("frame is not valid JSON")]
    InvalidJson,
    /// The schema version did not match.
    #[error("schema version {got} is not supported (expected {expected})")]
    SchemaMismatch {
        /// Version received.
        got: u16,
        /// Version supported.
        expected: u16,
    },
    /// The sequence was not strictly increasing.
    #[error("event sequence {got} is not after {last}")]
    StaleSequence {
        /// Last accepted sequence.
        last: u64,
        /// Sequence received.
        got: u64,
    },
    /// The command could not be serialized or encoded.
    #[error("frame could not be encoded")]
    Encode,
}

/// Tracks inbound sequence numbers so duplicates and stale frames are rejected.
#[derive(Debug, Default, Clone)]
pub struct SequenceTracker {
    last: Option<Sequence>,
}

impl SequenceTracker {
    /// Creates an empty tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self { last: None }
    }

    /// Returns the last accepted sequence.
    #[must_use]
    pub const fn last(&self) -> Option<Sequence> {
        self.last
    }

    /// Accepts `sequence` when it is strictly increasing.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError::StaleSequence`] for a duplicate or reordered frame.
    /// The tracker is not advanced on rejection.
    pub fn accept(&mut self, sequence: Sequence) -> Result<(), IpcError> {
        if let Some(last) = self.last
            && sequence.get() <= last.get()
        {
            return Err(IpcError::StaleSequence {
                last: last.get(),
                got: sequence.get(),
            });
        }
        self.last = Some(sequence);
        Ok(())
    }
}

/// Decodes and validates an inbound frame.
///
/// # Errors
///
/// Returns [`IpcError`] for an oversized, empty, malformed, wrong-version or
/// stale frame. A rejected frame never reaches command handling.
pub fn decode_command(
    frame: &[u8],
    tracker: &mut SequenceTracker,
) -> Result<CommandEnvelope, IpcError> {
    if frame.is_empty() {
        return Err(IpcError::EmptyFrame);
    }
    if frame.len() > MAX_IPC_FRAME_BYTES {
        return Err(IpcError::FrameTooLarge {
            size: frame.len(),
            limit: MAX_IPC_FRAME_BYTES,
        });
    } // Adjacently tagged enums ignore unknown keys next to the tag, so the two
    // enum levels are checked explicitly. Everything below them is a struct
    // with `deny_unknown_fields`.
    let raw: serde_json::Value =
        serde_json::from_slice(frame).map_err(|_| IpcError::InvalidJson)?;
    reject_extra_keys(&raw)?;
    let envelope: CommandEnvelope =
        serde_json::from_slice(frame).map_err(|_| IpcError::InvalidJson)?;
    if envelope.schema_version != SCHEMA_VERSION {
        return Err(IpcError::SchemaMismatch {
            got: envelope.schema_version,
            expected: SCHEMA_VERSION,
        });
    }
    tracker.accept(envelope.sequence)?;
    Ok(envelope)
}

/// Rejects unexpected keys at the two tagged enum levels.
///
/// A closed schema is only closed if an unexpected key is an error: serde's
/// adjacent tagging tolerates extra keys beside the tag, so they are rejected
/// here before the payload can reach command handling.
fn reject_extra_keys(value: &serde_json::Value) -> Result<(), IpcError> {
    let object = value.as_object().ok_or(IpcError::InvalidJson)?;
    let payload = object.get("payload").ok_or(IpcError::InvalidJson)?;
    let payload = payload.as_object().ok_or(IpcError::InvalidJson)?;
    if payload
        .keys()
        .any(|key| key != "domain" && key != "command")
    {
        return Err(IpcError::InvalidJson);
    }
    let command = payload.get("command").ok_or(IpcError::InvalidJson)?;
    let command = command.as_object().ok_or(IpcError::InvalidJson)?;
    if command.keys().any(|key| key != "op" && key != "args") {
        return Err(IpcError::InvalidJson);
    }
    Ok(())
}

/// Encodes one outbound event as a single JSONL line.
///
/// # Errors
///
/// Returns [`IpcError::Encode`] when the payload cannot be serialized.
pub fn encode_event(event: &EventEnvelope) -> Result<String, IpcError> {
    let mut line = serde_json::to_string(event).map_err(|_| IpcError::Encode)?;
    line.push('\n');
    Ok(line)
}

/// Returns the readiness controls as owned strings for the UI.
#[must_use]
pub fn readiness_controls_for(state: ReadinessState) -> Vec<String> {
    ba_domain::readiness_controls(state)
        .iter()
        .map(|control| (*control).to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ba_domain::IdError;

    fn frame(payload: &str) -> Vec<u8> {
        payload.as_bytes().to_vec()
    }

    fn envelope(command: &str) -> String {
        format!(
            r#"{{"schema_version":{SCHEMA_VERSION},"sequence":1,"at_unix_ms":1700000000000,"payload":{command}}}"#
        )
    }

    #[test]
    fn a_valid_command_is_accepted() {
        let mut tracker = SequenceTracker::new();
        let decoded = decode_command(
            &frame(&envelope(
                r#"{"domain":"agent","command":{"op":"PAUSE","args":{}}}"#,
            )),
            &mut tracker,
        )
        .unwrap();
        assert_eq!(decoded.payload.name(), "agent.pause");
        assert_eq!(tracker.last(), Some(Sequence::new(1)));
    }

    #[test]
    fn unknown_domains_operations_and_fields_are_rejected() {
        let cases = [
            r#"{"domain":"kernel","command":{"op":"PANIC"}}"#,
            r#"{"domain":"agent","command":{"op":"DELETE_EVERYTHING"}}"#,
            r#"{"domain":"agent","command":{"op":"PAUSE","extra":true}}"#,
            r#"{"domain":"agent","command":{"op":"PAUSE","args":{"extra":true}}}"#,
            r#"{"domain":"browser","command":{"op":"NAVIGATE","args":{"url":"https://example.com","authorized":true}}}"#,
        ];
        for payload in cases {
            let mut tracker = SequenceTracker::new();
            assert!(
                decode_command(&frame(&envelope(payload)), &mut tracker).is_err(),
                "{payload} must be rejected"
            );
        }
    }

    #[test]
    fn extra_top_level_fields_are_rejected() {
        let mut tracker = SequenceTracker::new();
        let payload = format!(
            r#"{{"schema_version":{SCHEMA_VERSION},"sequence":1,"at_unix_ms":0,"payload":{{"domain":"agent","command":{{"op":"PAUSE","args":{{}}}}}},"injected":true}}"#
        );
        assert!(decode_command(&frame(&payload), &mut tracker).is_err());
    }

    #[test]
    fn oversized_and_empty_frames_are_rejected_before_parsing() {
        let mut tracker = SequenceTracker::new();
        assert_eq!(decode_command(&[], &mut tracker), Err(IpcError::EmptyFrame));
        let huge = vec![b'x'; MAX_IPC_FRAME_BYTES + 1];
        assert!(matches!(
            decode_command(&huge, &mut tracker),
            Err(IpcError::FrameTooLarge { .. })
        ));
    }

    #[test]
    fn schema_and_sequence_are_enforced() {
        let mut tracker = SequenceTracker::new();
        let wrong_version = r#"{"schema_version":99,"sequence":1,"at_unix_ms":0,"payload":{"domain":"agent","command":{"op":"PAUSE","args":{}}}}"#;
        assert!(matches!(
            decode_command(&frame(wrong_version), &mut tracker),
            Err(IpcError::SchemaMismatch { got: 99, .. })
        ));
        assert_eq!(tracker.last(), None, "a rejected frame must not advance");

        decode_command(
            &frame(&envelope(
                r#"{"domain":"agent","command":{"op":"PAUSE","args":{}}}"#,
            )),
            &mut tracker,
        )
        .unwrap();
        let replayed = envelope(r#"{"domain":"agent","command":{"op":"PAUSE","args":{}}}"#);
        assert!(matches!(
            decode_command(&frame(&replayed), &mut tracker),
            Err(IpcError::StaleSequence { .. })
        ));
    }

    #[test]
    fn events_are_single_line_jsonl() {
        let event = EventEnvelope::new(
            Sequence::new(4),
            UnixMillis::from_millis(1_700_000_000_000),
            900,
            Some(TaskId::parse("task-1").unwrap()),
            None,
            EventPayload::Error {
                error_id: "BA-ERR-MODEL-001".to_owned(),
                attempted: "Browser-Agent tried to start this task.".to_owned(),
                observed: "The local model is not ready.".to_owned(),
                known: "No page action was proposed.".to_owned(),
                next_action: "Open Model status.".to_owned(),
                rule: None,
            },
        );
        let line = encode_event(&event).unwrap();
        assert!(line.ends_with('\n'));
        assert_eq!(line.matches('\n').count(), 1);
        let parsed: serde_json::Value = serde_json::from_str(line.trim_end()).unwrap();
        assert_eq!(parsed["schema_version"], SCHEMA_VERSION);
        assert_eq!(parsed["payload"]["kind"], "ERROR");
    }

    #[test]
    fn approval_arguments_are_fully_typed() {
        let payload = envelope(
            r#"{"domain":"agent","command":{"op":"APPROVE_ACTION","args":{"action_id":"action-4","digest":"0000000000000000000000000000000000000000000000000000000000000000","origin":"https://example.com","epoch":"epoch-2","frame":"frame-1"}}}"#,
        );
        let mut tracker = SequenceTracker::new();
        let decoded = decode_command(&frame(&payload), &mut tracker).unwrap();
        assert_eq!(decoded.payload.name(), "agent.approveAction");
        match decoded.payload {
            Command::Agent(AgentCommand::ApproveAction(args)) => {
                assert_eq!(args.origin.canonical(), "https://example.com");
            }
            other => panic!("unexpected command {other:?}"),
        }
    }

    #[test]
    fn bad_identifiers_in_commands_are_rejected() {
        let payload = envelope(
            r#"{"domain":"browser","command":{"op":"CLOSE_TAB","args":{"tab":"../etc"}}}"#,
        );
        let mut tracker = SequenceTracker::new();
        assert!(decode_command(&frame(&payload), &mut tracker).is_err());
        assert!(matches!(
            TabId::parse("../etc"),
            Err(IdError::InvalidCharacter)
        ));
    }

    #[test]
    fn readiness_controls_come_from_the_canonical_table() {
        assert_eq!(
            readiness_controls_for(ReadinessState::NotInstalled),
            vec!["Install model".to_owned()]
        );
        assert!(readiness_controls_for(ReadinessState::Ready).is_empty());
    }
}
