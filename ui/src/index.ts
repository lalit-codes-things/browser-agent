/**
 * Browser-Agent first-party UI public API.
 *
 * All app-owned UI is serif-first, uses the immutable seven-color palette with
 * approved surface-aware semantic tokens, and never ships app-owned icons
 * outside the Undo/Redo/Reload/Bookmarks/Downloads/three-dot whitelist.
 */
export {
  BrowserProvider,
  useBrowser,
  type BrowserState,
  type Tab,
  type TabId,
  type BrowserApi,
  type TabsApi,
  type BridgeApi,
  type NativeBridge,
  type BridgeEvent,
} from './domain';
export {
  AgentProvider,
  useAgent,
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
  type AgentReadinessStatus,
  type SecondaryViewName,
  type AgentPanelState,
  type TaskSummary,
  type TaskEvent,
  type AgentReadiness,
  type SecondaryView,
  type ApprovalDetails,
  type ApprovalAction,
  type TaskState,
  type WaitReason,
  type VerificationOutcome,
  type ActionClass,
} from './domain';
export { SecondaryViewProvider, useSecondaryView } from './domain';
export { SecondaryViewPortal } from './components/SecondaryViewPortal';
export { Button } from './components/Button';
export { TabStrip } from './components/TabStrip';
export { Toolbar } from './components/Toolbar';
export { BrowserView } from './components/BrowserView';
export { AgentPanel } from './components/AgentPanel';
