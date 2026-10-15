/**
 * Browser-Agent domain vocabulary.
 *
 * State machines and their copy are owned by the Rust core; this module names
 * what the UI is allowed to render and re-exports the validated wire views so a
 * component never re-declares a runtime shape. Values here cannot grant
 * authority: the policy layer decides what may happen.
 */
import type {
  ApprovalView,
  AuditStatusView,
  BrowserStateView,
  BudgetField,
  DownloadView,
  HandoffView,
  NoteView,
  ReadinessView,
  RuntimeErrorView,
  SkillView,
  TaskBudgetView,
  TaskEventView,
  TaskHistoryEntryView,
  TaskStatusView,
} from '../bridge/protocol';

export type {
  ApprovalView,
  AuditStatusView,
  BrowserStateView,
  BudgetField,
  DownloadView,
  HandoffView,
  NoteView,
  ReadinessView,
  RuntimeErrorView,
  SkillView,
  TaskBudgetView,
  TaskEventView,
  TaskHistoryEntryView,
  TaskStatusView,
};

/** Task lifecycle states, as declared by the Rust orchestrator (Section 12.2). */
export type TaskState =
  | 'CREATED'
  | 'NORMALIZING'
  | 'OBSERVING'
  | 'PROPOSING'
  | 'POLICY_CHECK'
  | 'EXECUTING'
  | 'VERIFYING'
  | 'WAITING_FOR_USER'
  | 'USER_TAKEOVER'
  | 'PAUSED'
  | 'BLOCKED'
  | 'FAILED'
  | 'CANCELLED'
  | 'COMPLETED';

export type WaitReason = 'ACTION_APPROVAL' | 'USER_INPUT' | 'AUTHENTICATION' | 'TASK_BUDGET';

export type VerificationOutcome =
  'VERIFIED_SUCCESS' | 'LIKELY_SUCCESS' | 'UNKNOWN' | 'LIKELY_FAILURE' | 'VERIFIED_FAILURE';

export type ActionClass = 'READ' | 'REVERSIBLE_WRITE' | 'IRREVERSIBLE' | 'UNKNOWN';

/**
 * Capability readiness vocabulary, transcribed from [BA-READY-001]. Readiness
 * is an independent field from task lifecycle, task budget and verification
 * outcome; the four must never be merged into one status.
 */
export type AgentReadinessStatus =
  | 'NOT_INSTALLED'
  | 'INSTALLING'
  | 'INTEGRITY_FAILED'
  | 'INCOMPATIBLE'
  | 'LOADING'
  | 'READY'
  | 'PARKED'
  | 'RUNNER_FAILED'
  | 'RESOURCE_STOPPED';

export interface AgentReadiness {
  status: AgentReadinessStatus;
  /** Canonical label reported by the runtime, or the local table copy. */
  label: string;
  /** Measured detail from the runtime; never a fabricated progress claim. */
  message: string;
}

/** One tab as reported by the managed host: identity plus its current title. */
export interface TabSummary {
  id: string;
  title: string;
}

/** Optional secondary screens. Every one renders real runtime content. */
export type SecondaryView =
  'about' | 'history' | 'downloads' | 'model' | 'notes' | 'audit' | 'settings';

export interface AgentPanelState {
  readiness: AgentReadiness;
  /** Latest authoritative task status, or `null` when no task is active. */
  status: TaskStatusView | null;
  /** Task activity as reported by the runtime, oldest first. */
  activity: TaskEventView[];
  /** Terminal report, when the runtime reported one. */
  ended: {
    task_id: string;
    state: string;
    state_label: string;
    verification: string | null;
    verification_label: string | null;
    detail: string;
  } | null;
  /** A structured action approval is pending. */
  approval: ApprovalView | null;
  /** The user must complete a step the agent may not perform. */
  handoff: HandoffView | null;
  /** Most recent four-part error from the runtime. */
  error: RuntimeErrorView | null;
  /** True while a non-terminal task is in flight ([BA-WIN-002]). */
  composerLocked: boolean;
}
