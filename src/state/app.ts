// Frontend application state.
//
// This is a projection of typed backend IPC events. It does not invent
// security truth. Where a backend field is absent, we keep it absent or
// express it as an explicit unknown state (C-01 discipline).

export interface AppState {
  eventLog: AppEvent[];
  task: TaskSnapshot | null;
  browser: BrowserSnapshot | null;
  navigation: NavigationSnapshot | null;
  perception: PerceptionSnapshot | null;
  model: ModelSnapshot | null;
  policy: PolicySnapshot | null;
  execution: ExecutionSnapshot | null;
  verification: VerificationSnapshot | null;
  authorization: AuthorizationSnapshot | null;
  agentCursor: AgentCursorSnapshot | null;
  humanTakeover: HumanTakeoverSnapshot | null;
  actionDurable: ActionDurableSnapshot | null;
  ui: UIState;
}

export interface UIState {
  panels: Record<string, boolean>;
}

export interface TaskSnapshot {
  id: string;
  status: TaskStatus;
  stepLabel: string;
  authority?: string;
  progress?: string;
}

export type TaskStatus =
  | "PENDING"
  | "RUNNING"
  | "VERIFIED"
  | "LIKELY_SUCCESS"
  | "UNKNOWN"
  | "FAILED"
  | "ABORTED"
  | "PARKED";

export interface BrowserSnapshot {
  available: boolean;
  state: BrowserRuntimeState;
  reason?: string;
}

export type BrowserRuntimeState =
  | "READY"
  | "UNAVAILABLE"
  | "CRASHED"
  | "STOPPED";

export interface NavigationSnapshot {
  taskId: string;
  origin?: string;
  url?: string;
  loaderId?: string;
  frameId?: string;
  loading: boolean;
}

export interface PerceptionSnapshot {
  taskId: string;
  epoch: number;
  frameId: string;
  loaderId: string;
  actionableCount: number;
  boundedExceeded: boolean;
}

export interface ModelSnapshot {
  modelId: string;
  residency: ModelResidency;
  verifiedAtLoad: boolean;
}

export type ModelResidency = "LOADED" | "PARKED" | "UNLOADED";

export interface PolicySnapshot {
  taskId: string;
  actionClass: string;
  tier: string;
  verdict: string;
  reason: string;
}

export interface ExecutionSnapshot {
  taskId: string;
  action: string;
  success: boolean;
  failureReason?: string;
  postState?: string;
  epoch: number;
}

export interface VerificationSnapshot {
  taskId: string;
  outcome: VerificationOutcome;
  evidenceCount: number;
  timestamp?: string;
}

export type VerificationOutcome =
  | "VERIFIED_SUCCESS"
  | "LIKELY_SUCCESS"
  | "UNKNOWN"
  | "LIKELY_FAILURE"
  | "VERIFIED_FAILURE";

export interface AuthorizationSnapshot {
  taskId: string;
  tier: string;
  summary: string;
}

export interface AgentCursorSnapshot {
  active: boolean;
  state: AgentCursorState;
  targetFrameId?: string;
  targetLoaderId?: string;
  x?: number;
  y?: number;
  reason?: string;
}

export type AgentCursorState =
  | "IDLE"
  | "MOVING"
  | "CLICKING"
  | "BLOCKED"
  | "HANDING_OFF";

export interface HumanTakeoverSnapshot {
  taskId: string;
  mode: HumanTakeoverMode;
  reason?: string;
}

export type HumanTakeoverMode =
  | "IDLE"
  | "HANDOFF_REQUESTED"
  | "HANDOFF_ACTIVE"
  | "HANDOFF_DENIED"
  | "HANDOFF_EXPIRED";

export interface ActionDurableSnapshot {
  taskId: string;
  durableId: string;
  durableState: ActionDurableState;
  actionCommitmentHash: string;
}

export type ActionDurableState =
  | "UNKNOWN"
  | "SUBMITTED"
  | "WAITING_FOR_EXTERNAL_AUTH"
  | "PROCESSING"
  | "READY"
  | "RECONCILED"
  | "RESOLVED";

export type AppEvent =
  | { type: "TASK_STATE"; payload: TaskStateEvent }
  | { type: "BROWSER_RUNTIME_STATE"; payload: BrowserRuntimeStateEvent }
  | { type: "NAVIGATION_STATE"; payload: NavigationStateEvent }
  | { type: "PERCEPTION_STATE"; payload: PerceptionStateEvent }
  | { type: "ACTION_PROPOSAL"; payload: ActionProposalEvent }
  | { type: "POLICY_DECISION"; payload: PolicyDecisionEvent }
  | { type: "EXECUTION_RESULT"; payload: ExecutionResultEvent }
  | { type: "VERIFICATION_OUTCOME"; payload: VerificationOutcomeEvent }
  | { type: "AUTHORIZATION_REQUIRED"; payload: AuthorizationRequiredEvent }
  | { type: "AGENT_CURSOR_STATE"; payload: AgentCursorStateEvent }
  | { type: "HUMAN_TAKEOVER_STATE"; payload: HumanTakeoverStateEvent }
  | { type: "ACTION_DURABLE_STATE"; payload: ActionDurableStateEvent };

export interface TaskStateEvent {
  task_id: string;
  status: TaskStatus;
  step_label: string;
}

export interface BrowserRuntimeStateEvent {
  available: boolean;
  state: BrowserRuntimeState;
  reason?: string;
}

export interface NavigationStateEvent {
  task_id: string;
  origin?: string;
  url?: string;
  loader_id?: string;
  frame_id?: string;
  loading: boolean;
}

export interface PerceptionStateEvent {
  task_id: string;
  epoch: number;
  frame_id: string;
  loader_id: string;
  actionable_count: number;
  bounded_exceeded: boolean;
}

export interface ActionProposalEvent {
  task_id: string;
  action: string;
  action_class: string;
  semantic_reference?: string;
  epoch: number;
}

export interface PolicyDecisionEvent {
  task_id: string;
  action_class: string;
  tier: string;
  verdict: string;
  reason: string;
}

export interface ExecutionResultEvent {
  task_id: string;
  action: string;
  success: boolean;
  failure_reason?: string;
  post_state?: string;
  epoch: number;
}

export interface VerificationOutcomeEvent {
  task_id: string;
  outcome: VerificationOutcome;
  evidence_count: number;
  timestamp: string;
}

export interface AuthorizationRequiredEvent {
  task_id: string;
  tier: string;
  summary: string;
}

export interface AgentCursorStateEvent {
  active: boolean;
  state: AgentCursorState;
  target_frame_id?: string;
  target_loader_id?: string;
  x?: number;
  y?: number;
  reason?: string;
}

export interface HumanTakeoverStateEvent {
  task_id: string;
  mode: HumanTakeoverMode;
  reason?: string;
}

export interface ActionDurableStateEvent {
  task_id: string;
  durable_id: string;
  durable_state: ActionDurableState;
  action_commitment_hash: string;
}

export function reduceEvent(state: AppState, event: AppEvent): AppState {
  const next: AppState = {
    ...state,
    eventLog:
      state.eventLog.length >= 500
        ? [...state.eventLog.slice(-499), event]
        : [...state.eventLog, event],
  };

  switch (event.type) {
    case "TASK_STATE": {
      return {
        ...next,
        task: {
          id: event.payload.task_id,
          status: event.payload.status,
          stepLabel: event.payload.step_label,
        },
      };
    }
    case "BROWSER_RUNTIME_STATE": {
      return {
        ...next,
        browser: {
          available: event.payload.available,
          state: event.payload.state,
          reason: event.payload.reason,
        },
      };
    }
    case "NAVIGATION_STATE": {
      return {
        ...next,
        navigation: {
          taskId: event.payload.task_id,
          origin: event.payload.origin,
          url: event.payload.url,
          loaderId: event.payload.loader_id,
          frameId: event.payload.frame_id,
          loading: event.payload.loading,
        },
      };
    }
    case "PERCEPTION_STATE": {
      return {
        ...next,
        perception: {
          taskId: event.payload.task_id,
          epoch: event.payload.epoch,
          frameId: event.payload.frame_id,
          loaderId: event.payload.loader_id,
          actionableCount: event.payload.actionable_count,
          boundedExceeded: event.payload.bounded_exceeded,
        },
      };
    }
    case "ACTION_PROPOSAL": {
      return next;
    }
    case "POLICY_DECISION": {
      return {
        ...next,
        policy: {
          taskId: event.payload.task_id,
          actionClass: event.payload.action_class,
          tier: event.payload.tier,
          verdict: event.payload.verdict,
          reason: event.payload.reason,
        },
      };
    }
    case "EXECUTION_RESULT": {
      return {
        ...next,
        execution: {
          taskId: event.payload.task_id,
          action: event.payload.action,
          success: event.payload.success,
          failureReason: event.payload.failure_reason,
          postState: event.payload.post_state,
          epoch: event.payload.epoch,
        },
      };
    }
    case "VERIFICATION_OUTCOME": {
      return {
        ...next,
        verification: {
          taskId: event.payload.task_id,
          outcome: event.payload.outcome,
          evidenceCount: event.payload.evidence_count,
          timestamp: event.payload.timestamp,
        },
      };
    }
    case "AUTHORIZATION_REQUIRED": {
      return {
        ...next,
        authorization: {
          taskId: event.payload.task_id,
          tier: event.payload.tier,
          summary: event.payload.summary,
        },
      };
    }
    case "AGENT_CURSOR_STATE": {
      return {
        ...next,
        agentCursor: {
          active: event.payload.active,
          state: event.payload.state,
          targetFrameId: event.payload.target_frame_id,
          targetLoaderId: event.payload.target_loader_id,
          x: event.payload.x,
          y: event.payload.y,
          reason: event.payload.reason,
        },
      };
    }
    case "HUMAN_TAKEOVER_STATE": {
      return {
        ...next,
        humanTakeover: {
          taskId: event.payload.task_id,
          mode: event.payload.mode,
          reason: event.payload.reason,
        },
      };
    }
    case "ACTION_DURABLE_STATE": {
      return {
        ...next,
        actionDurable: {
          taskId: event.payload.task_id,
          durableId: event.payload.durable_id,
          durableState: event.payload.durable_state,
          actionCommitmentHash: event.payload.action_commitment_hash,
        },
      };
    }
    default:
      return next;
  }
}

export const initialState: AppState = {
  eventLog: [],
  task: null,
  browser: null,
  navigation: null,
  perception: null,
  model: null,
  policy: null,
  execution: null,
  verification: null,
  authorization: null,
  agentCursor: null,
  humanTakeover: null,
  actionDurable: null,
  ui: { panels: {} },
};
