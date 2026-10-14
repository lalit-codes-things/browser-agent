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

export interface Scheme {
  scheme: 'http' | 'https' | 'file' | 'browser-agent';
}

export interface UrlParts {
  scheme: Scheme;
  host: string;
  port: number | null;
  path: string;
  query: string;
  fragment: string;
}

export interface SecurityState {
  state: 'https' | 'http' | 'mixed-content' | 'unknown';
}

export interface ApprovalState {
  state: 'idle' | 'pending' | 'approved' | 'rejected';
}

export interface ApprovalAction {
  action: 'click' | 'type' | 'navigate' | 'scroll' | 'screenshot' | 'read';
}

export interface AgentReadinessStatus {
  status: 'NOT_INSTALLED' | 'INSTALLING' | 'READY' | 'ERROR';
}

export interface AgentReadiness {
  status: AgentReadinessStatus;
  message: string;
}

export interface Tab {
  id: string;
  url: string;
  title: string;
  status: 'loading' | 'ready' | 'blocked' | 'error';
  lastUpdated: number;
  observationEpoch: string;
}

export type TabId = string;

export interface BrowserState {
  windows: number;
  activeTabId: TabId | null;
  tabs: readonly Tab[];
  currentUrl: string;
  securityState: SecurityState;
  loading: boolean;
  canGoBack: boolean;
  canGoForward: boolean;
  agentReady: boolean;
  bridgeHandle: number;
}

export interface TaskAction {
  actionType: ActionClass;
  targetOrigin: string;
  targetSelector: string | null;
  targetText?: string;
}

export interface ApprovalDetails {
  actionId: string;
  action: TaskAction;
}

export interface TaskEvent {
  taskId: string;
  revision: number;
  sequence: number;
  state: TaskState;
  waitReason: WaitReason | null;
  timestamp: number;
  actionLabel: string | null;
  detail: string;
}

export interface TaskSummary {
  taskId: string;
  originalText: string;
  state: TaskState;
  waitReason: WaitReason | null;
  currentAction: string | null;
  elapsedMs: number;
  events: readonly TaskEvent[];
  verification: VerificationOutcome | null;
}

export interface NativeBridge {
  bridgeHandle: number;
  approvalState: { actionId: string } | null;
  onEvent: (listener: (event: BridgeEvent) => void) => () => void;
  navigate: (url: string) => Promise<void>;
  back: () => Promise<void>;
  forward: () => Promise<void>;
  reload: () => Promise<void>;
  stop: () => Promise<void>;
  focusAddress: () => Promise<void>;
  tabsCreate: (url: string, title: string) => Promise<string>;
  tabsClose: (tabId: string) => Promise<void>;
  tabsActivate: (tabId: string) => Promise<void>;
  agentSubmitTask: (taskText: string) => Promise<string>;
  agentPause: () => Promise<void>;
  agentCancel: () => Promise<void>;
  agentTakeOver: () => Promise<void>;
  agentResume: () => Promise<void>;
  agentRespondToApproval: (actionId: string, approved: boolean) => Promise<void>;
}

export interface BridgeEvent {
  kind: string;
  payload: unknown;
}

export interface BrowserApi {
  state: BrowserState;
  openTab: (url: string, title: string) => Tab;
  closeTab: (id: TabId) => void;
  navigateTab: (id: TabId, url: string) => void;
  selectTab: (id: TabId) => void;
  activateTab: (id: TabId) => void;
  back: () => Promise<void>;
  forward: () => Promise<void>;
  reload: () => Promise<void>;
  focusAddress: () => Promise<void>;
}

export interface TabsApi {
  create: (url: string, title: string) => Tab;
  remove: (id: TabId) => void;
  navigate: (id: TabId, url: string) => void;
  activeTabId: Tab['id'] | null;
}

export interface BridgeApi {
  openTab: (url: string, title: string) => Tab;
  closeTab: (tabId: string) => void;
  activateTab: (tabId: string) => void;
  navigate: (tabId: string, url: string) => void;
}

export type SecondaryView = string;

export interface AgentPanelState {
  readiness: AgentReadiness;
  task: TaskSummary | null;
  composerLocked: boolean;
  focusOnCancel: boolean;
  approval: ApprovalDetails | null;
}
