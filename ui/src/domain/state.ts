import {
  AgentPanelState,
  AgentReadiness,
  TaskState,
  WaitReason,
  TaskEvent,
  TaskSummary,
  BrowserState,
  VerificationOutcome,
} from './types';

/**
 * Runtime state defaults owned by the Rust orchestrator.
 * No frontend timer or canned text may certify inference, navigation, download
 * or policy state. React renders the authoritative state emitted by the runtime.
 */

export function createTaskEvent(
  taskId: string,
  revision: number,
  sequence: number,
  state: TaskState,
  waitReason: WaitReason | null,
  timestamp: number,
  actionLabel: string | null,
  _detail: string,
): TaskEvent {
  return {
    taskId,
    revision,
    sequence,
    state,
    waitReason,
    timestamp,
    actionLabel,
    detail: actionLabel ?? '',
  };
}

export function createTaskSummary(
  taskId: string,
  originalText: string,
  state: TaskState,
  waitReason: WaitReason | null,
  currentAction: string | null,
  elapsedMs: number,
  events: readonly TaskEvent[],
  verification: VerificationOutcome | null,
): TaskSummary {
  return {
    taskId,
    originalText,
    state,
    waitReason,
    currentAction,
    elapsedMs,
    events,
    verification,
  };
}

export function createBrowserState(): BrowserState {
  return {
    windows: 1,
    activeTabId: null,
    tabs: [],
    currentUrl: 'browser-agent://newtab/',
    securityState: { state: 'https' },
    loading: false,
    canGoBack: false,
    canGoForward: false,
    agentReady: false,
    bridgeHandle: 0,
  };
}

/**
 * No model can be loaded until the managed browser host and the local runner
 * exist. Absence is reported as the canonical `NOT_INSTALLED` readiness state
 * with measured detail, never as a task failure ([BA-READY-001]).
 */
export function createAgentReadiness(): AgentReadiness {
  return {
    status: 'NOT_INSTALLED',
    message: 'Model not installed',
  };
}

export function createAgentPanelState(): AgentPanelState {
  return {
    readiness: createAgentReadiness(),
    task: null,
    composerLocked: false,
    focusOnCancel: true,
    approval: null,
  };
}
