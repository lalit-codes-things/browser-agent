/** Browser-Agent domain model. These types are the single source of truth for
 * the native/frontend contract. Values cannot grant authority on their own;
 * the Rust policy layer owns authorization and this data only reflects runtime
 * state observed through the native bridge. */
export type {
  Scheme,
  TaskState,
  WaitReason,
  Tab,
  TabId,
  UrlParts,
  BrowserState,
  ApprovalState,
  TaskEvent,
  TaskSummary,
  VerificationOutcome,
  ActionClass,
  ApprovalDetails,
  ApprovalAction,
  AgentReadiness,
  AgentPanelState,
  NativeBridge,
  BridgeEvent,
  BrowserApi,
  TabsApi,
  BridgeApi,
  SecondaryView,
  AgentReadinessStatus,
} from './types';

export { NATIVE_BRIDGE_ABSENT_HANDLE } from './types';

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

/** Runtime state defaults (owned by the Rust orchestrator). */
export {
  createBrowserState,
  createAgentReadiness,
  createAgentPanelState,
  createTaskEvent,
  createTaskSummary,
} from './state';

/** React hooks for the permanent Agent panel. */
export {
  AgentProvider,
  useAgent,
  MAX_TASK_CODE_POINTS,
  countCodePoints,
  RUNTIME_NOT_CONNECTED_COPY,
  RUNTIME_NOT_CONNECTED_DETAIL,
  TASK_NOT_STARTED_COPY,
  COMPOSER_OVER_LIMIT_COPY,
  type AgentContextValue,
} from './useAgent';

/** React context for the managed browser. */
export { BrowserProvider, useBrowser } from './useBrowser';

/** React hook for tab collection ownership. */
export { useTabs } from './useTabs';

/** React context for optional secondary-screen overlays. */
export {
  SecondaryViewProvider,
  useSecondaryView,
  type SecondaryViewName,
} from './useSecondaryView';
