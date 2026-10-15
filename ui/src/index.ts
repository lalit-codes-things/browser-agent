/**
 * Browser-Agent first-party UI public API.
 *
 * All app-owned UI is serif-first, uses the immutable seven-color palette with
 * approved surface-aware semantic tokens, and never ships app-owned icons
 * outside the Undo/Redo/Reload/Bookmarks/Downloads/three-dot whitelist.
 */
export {
  RuntimeProvider,
  BrowserProvider,
  AgentProvider,
  SecondaryViewProvider,
  useRuntime,
  useBrowser,
  useAgent,
  useRuntimeData,
  useSecondaryView,
  useShortcuts,
  SHORTCUTS,
  shortcutText,
  MAX_TASK_CODE_POINTS,
  countCodePoints,
  TASK_STATES,
  WAIT_REASONS,
  READINESS_STATES,
  READINESS_COPY,
  VERIFICATION_OUTCOMES,
  VERIFICATION_COPY,
  taskLifecycleLabel,
  verificationCopy,
  isTerminalTaskState,
  hasValidWaitReason,
  type AgentContextValue,
  type AgentPanelState,
  type AgentReadiness,
  type AgentReadinessStatus,
  type ApprovalView,
  type AuditStatusView,
  type BrowserStateView,
  type BudgetField,
  type DownloadView,
  type HandoffView,
  type NoteView,
  type ReadinessView,
  type RuntimeContextValue,
  type RuntimeErrorView,
  type SecondaryView,
  type SecondaryViewName,
  type SkillView,
  type TabSummary,
  type TaskBudgetView,
  type TaskEventView,
  type TaskHistoryEntryView,
  type TaskState,
  type TaskStatusView,
  type VerificationOutcome,
  type WaitReason,
  type ActionClass,
} from './domain';
export { SecondaryViewPortal } from './components/SecondaryViewPortal';
export { Button } from './components/Button';
export { TabStrip } from './components/TabStrip';
export { Toolbar } from './components/Toolbar';
export { BrowserView } from './components/BrowserView';
export { AgentPanel } from './components/AgentPanel';

/** The validated runtime bridge, for hosts and integration tests. */
export {
  SCHEMA_VERSION,
  MAX_IPC_FRAME_BYTES,
  EVENT_KINDS,
  decodeEventLine,
  encodeCommandLine,
  createRuntimeClient,
  attachHostChannel,
  isHostChannel,
  commands,
  type CommandBody,
  type DecodeFailure,
  type DecodeResult,
  type EventKind,
  type HostChannel,
  type RuntimeClient,
  type RuntimeEvent,
  type RuntimeSnapshot,
} from './bridge';
