/**
 * Browser-Agent domain layer.
 *
 * The vocabulary here mirrors the Rust schema, and the hooks render state that
 * arrived as validated runtime events. Nothing in this layer can grant
 * authority: the policy layer decides, the orchestrator advances, the verifier
 * concludes and React renders.
 */
export type {
  ActionClass,
  AgentPanelState,
  AgentReadiness,
  AgentReadinessStatus,
  ApprovalView,
  AuditStatusView,
  BrowserStateView,
  BudgetField,
  DownloadView,
  HandoffView,
  NoteView,
  ReadinessView,
  RuntimeErrorView,
  SecondaryView,
  SkillView,
  TabSummary,
  TaskBudgetView,
  TaskEventView,
  TaskHistoryEntryView,
  TaskState,
  TaskStatusView,
  VerificationOutcome,
  WaitReason,
} from './types';

/** Canonical Section 2.9.1 / 2.9.2 / 15.5 copy contract. Components render
 * these strings instead of inventing their own status vocabulary. */
export {
  TASK_STATES,
  WAIT_REASONS,
  READINESS_STATES,
  READINESS_COPY,
  VERIFICATION_OUTCOMES,
  VERIFICATION_COPY,
  VERIFICATION_NOT_RUN_COPY,
  INVALID_WAIT_STATE_COPY,
  isTerminalTaskState,
  isWaitReason,
  hasValidWaitReason,
  taskLifecycleLabel,
  verificationCopy,
  type VerificationCopy,
} from './copy';

/** Projection from validated runtime events to renderable panel state. */
export {
  UNKNOWN_READINESS_COPY,
  asReadinessStatus,
  asTaskState,
  createAgentPanelState,
  createAgentReadiness,
  panelStateFromSnapshot,
  readinessFromView,
} from './state';

/** React context for the connection to the Rust application core. */
export { RuntimeProvider, useRuntime, type RuntimeContextValue } from './useRuntime';

/** React context for the managed browser. */
export { BrowserProvider, useBrowser } from './useBrowser';

/** Commands and state for the secondary screens. */
export { useRuntimeData } from './useRuntimeData';

/** Core shortcut baseline (Section 18.2). */
export {
  SHORTCUTS,
  UNIMPLEMENTED_SHORTCUTS,
  isMacPlatform,
  shortcutText,
  useShortcuts,
  type ShortcutDefinition,
  type ShortcutId,
} from './useShortcuts';

/** React context for optional secondary-screen overlays. */
export {
  SecondaryViewProvider,
  useSecondaryView,
  type SecondaryViewName,
} from './useSecondaryView';

/** React context for the permanent Agent panel. */
export {
  AgentProvider,
  useAgent,
  MAX_TASK_CODE_POINTS,
  countCodePoints,
  RUNTIME_NOT_CONNECTED_COPY,
  RUNTIME_NOT_CONNECTED_DETAIL,
  TASK_NOT_STARTED_COPY,
  NO_MANAGED_WINDOW_COPY,
  TASK_ALREADY_RUNNING_COPY,
  COMPOSER_OVER_LIMIT_COPY,
  type AgentContextValue,
} from './useAgent';
