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
} from './types';

/** Runtime state defaults (owned by the Rust orchestrator). */
export {
  createBrowserState,
  createAgentReadiness,
  createAgentPanelState,
  createTaskEvent,
  createTaskSummary,
} from './state';

/** React hooks for the permanent Agent panel. */
export { AgentProvider, useAgent } from './useAgent';

/** React context for the managed browser. */
export { BrowserProvider, useBrowser } from './useBrowser';

/** React hook for tab collection ownership. */
export { useTabs } from './useTabs';

/** React context for optional secondary-screen overlays. */
export { SecondaryViewProvider, useSecondaryView } from './useSecondaryView';
