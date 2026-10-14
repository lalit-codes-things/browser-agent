import {
  AgentReadiness,
  TaskState,
  WaitReason,
  TaskEvent,
  TaskSummary,
  BrowserState,
  Tab,
  TabId,
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

export function createAgentReadiness(): AgentReadiness {
  return {
    status: { status: 'NOT_INSTALLED' },
    message: 'Model not installed',
  };
}

export interface AgentPanelState {
  readiness: AgentReadiness;
  task: TaskSummary | null;
  composerLocked: boolean;
  focusOnCancel: boolean;
  approval: null;
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

/**
 * The eager local bridge methods exist so first interaction is not blocked
 * (BA-BRIDGE-001). Optional secondary screens may lazy-load.
 */
export interface BrowserApi {
  state: BrowserState;
  openTab: (url: string, title: string) => Tab;
  closeTab: (id: TabId) => void;
  navigateTab: (id: TabId, url: string) => void;
  selectTab: (id: TabId) => void;
  activateTab: (id: TabId) => void;
}

export interface TabsApi {
  create: (url: string, title: string) => Tab;
  remove: (id: TabId) => void;
  navigate: (id: TabId, url: string) => void;
  activeTabId: Tab['id'] | null;
}
