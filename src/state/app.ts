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
  egress: EgressSnapshot | null;
  policy: PolicySnapshot | null;
  execution: ExecutionSnapshot | null;
  verification: VerificationSnapshot | null;
  authorization: AuthorizationSnapshot | null;
  agentCursor: AgentCursorSnapshot | null;
  humanTakeover: HumanTakeoverSnapshot | null;
  actionDurable: ActionDurableSnapshot | null;
  ui: UIState;
}

export type ScreenName =
  | "CONSOLE"
  | "INTERVENTION"
  | "POLICY"
  | "SKILLS"
  | "AUDIT"
  | "VAULT"
  | "MODEL"
  | "NETWORK"
  | "QUARANTINE"
  | "PROFILES"
  | "SETTINGS"
  | "STOPPED";

export interface UIState {
  currentScreen: ScreenName;
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
  quantization: string;
  residency: ModelResidency;
  availability: ModelAvailability;
  sha256: string;
  reason?: string;
}

export type ModelResidency = "LOADED" | "PARKED" | "UNLOADED";

export type ModelAvailability =
  | "VERIFIED_LOADED"
  | "PRESENT_UNVERIFIED"
  | "UNAVAILABLE";

export interface EgressSnapshot {
  mode: "ENFORCED" | "DISABLED";
  enforcementActive: boolean;
  reason?: string;
}

export interface ModelStateEvent {
  model_id: string;
  quantization: string;
  residency: ModelResidency;
  availability: ModelAvailability;
  sha256: string;
  reason?: string;
}

export interface EgressStateEvent {
  mode: "ENFORCED" | "DISABLED";
  enforcement_active: boolean;
  reason?: string;
}

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
  commitmentHash: string;
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
  | { type: "AUTHORIZATION_RESOLVED"; payload: AuthorizationResolvedEvent }
  | { type: "AGENT_CURSOR_STATE"; payload: AgentCursorStateEvent }
  | { type: "HUMAN_TAKEOVER_STATE"; payload: HumanTakeoverStateEvent }
  | { type: "ACTION_DURABLE_STATE"; payload: ActionDurableStateEvent }
  | { type: "MODEL_STATE"; payload: ModelStateEvent }
  | { type: "EGRESS_STATE"; payload: EgressStateEvent }
  | { type: "SET_SCREEN"; payload: ScreenName }
  | { type: "SET_PANEL"; payload: { panel: string; open: boolean } };

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
  commitment_hash: string;
}

export type AuthorizationDecision = "APPROVED" | "DENIED";

export interface AuthorizationResolvedEvent {
  task_id: string;
  decision: AuthorizationDecision;
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
      // Stale events from a different task are ignored, except PENDING,
      // which marks acceptance of a new task and switches projection.
      const adopting = Boolean(state.task && state.task.id !== event.payload.task_id);
      if (adopting && event.payload.status !== "PENDING") {
        return next;
      }
      return {
        ...next,
        task: {
          id: event.payload.task_id,
          status: event.payload.status,
          stepLabel: event.payload.step_label,
        },
        // Adopting a new task: drop the previous task's security-relevant
        // snapshots so nothing stale can be rendered against it.
        ...(adopting
          ? {
              navigation: null,
              perception: null,
              policy: null,
              execution: null,
              verification: null,
              authorization: null,
              agentCursor: null,
              humanTakeover: null,
              actionDurable: null,
            }
          : {}),
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
      if (state.task && state.task.id !== event.payload.task_id) {
        return next;
      }
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
      if (state.task && state.task.id !== event.payload.task_id) {
        return next;
      }
      // Epochs are backend-issued and monotonic per task; a delayed event
      // from an older epoch never regresses the projected state.
      if (state.perception && event.payload.epoch < state.perception.epoch) {
        return next;
      }
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
      if (state.task && state.task.id !== event.payload.task_id) {
        return next;
      }
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
      if (state.task && state.task.id !== event.payload.task_id) {
        return next;
      }
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
      if (state.task && state.task.id !== event.payload.task_id) {
        return next;
      }
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
      // Only backend-issued requests open the interlock. Stale requests
      // from an older task never replace the current task's request.
      if (state.task && state.task.id !== event.payload.task_id) {
        return next;
      }
      return {
        ...next,
        authorization: {
          taskId: event.payload.task_id,
          tier: event.payload.tier,
          summary: event.payload.summary,
          commitmentHash: event.payload.commitment_hash,
        },
      };
    }
    case "AUTHORIZATION_RESOLVED": {
      // The backend resolved the request; the interlock closes only on
      // the authoritative resolution event.
      if (state.authorization && state.authorization.taskId !== event.payload.task_id) {
        return next;
      }
      return { ...next, authorization: null };
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
      if (state.task && state.task.id !== event.payload.task_id) {
        return next;
      }
      return {
        ...next,
        humanTakeover: {
          taskId: event.payload.task_id,
          mode: event.payload.mode,
          reason: event.payload.reason,
        },
      };
    }
    case "MODEL_STATE": {
      return {
        ...next,
        model: {
          modelId: event.payload.model_id,
          quantization: event.payload.quantization,
          residency: event.payload.residency,
          availability: event.payload.availability,
          sha256: event.payload.sha256,
          reason: event.payload.reason,
        },
      };
    }
    case "EGRESS_STATE": {
      return {
        ...next,
        egress: {
          mode: event.payload.mode,
          enforcementActive: event.payload.enforcement_active,
          reason: event.payload.reason,
        },
      };
    }
    case "ACTION_DURABLE_STATE": {
      if (state.task && state.task.id !== event.payload.task_id) {
        return next;
      }
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
    case "SET_SCREEN": {
      return {
        ...next,
        ui: {
          ...next.ui,
          currentScreen: event.payload,
        },
      };
    }
    case "SET_PANEL": {
      return {
        ...next,
        ui: {
          ...next.ui,
          panels: {
            ...next.ui.panels,
            [event.payload.panel]: event.payload.open,
          },
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
  egress: null,
  policy: null,
  execution: null,
  verification: null,
  authorization: null,
  agentCursor: null,
  humanTakeover: null,
  actionDurable: null,
  ui: { currentScreen: "CONSOLE", panels: {} },
};
