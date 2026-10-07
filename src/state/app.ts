// Frontend application state.
//
// This is a projection of typed backend IPC events. It does not invent
// security truth. Where a backend field is absent, we keep it absent or
// express it as an explicit unknown state (C-01 discipline).
import { SubmitTaskRequest } from "../ipc/schemas";

export interface AppState {
  eventLog: AppEvent[];
  task: TaskSnapshot | null;
  model: ModelSnapshot | null;
  policy: PolicySnapshot | null;
  verification: VerificationSnapshot | null;
  authorization: AuthorizationSnapshot | null;
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

export interface ModelSnapshot {
  modelId: string;
  residency: ModelResidency;
  verifiedAtLoad: boolean;
}

export type ModelResidency = "LOADED" | "PARKED" | "UNLOADED";

export interface PolicySnapshot {
  actionClass: string;
  tier: string;
  verdict: string;
  reason: string;
}

export interface VerificationSnapshot {
  outcome: string;
  evidenceCount: number;
  timestamp?: string;
}

export interface AuthorizationSnapshot {
  tier: string;
  summary: string;
}

export type AppEvent =
  | { type: "TASK_STATE"; payload: { taskId: string; status: TaskStatus; stepLabel: string } }
  | { type: "MODEL_STATE"; payload: { modelId: string; residency: ModelResidency; verifiedAtLoad: boolean } }
  | { type: "POLICY_DECISION"; payload: { actionClass: string; tier: string; verdict: string; reason: string } }
  | { type: "VERIFICATION_OUTCOME"; payload: { outcome: string; evidenceCount: number; timestamp?: string } }
  | { type: "AUTHORIZATION_REQUIRED"; payload: { tier: string; summary: string } };

export function reduceEvent(state: AppState, event: AppEvent): AppState {
  const next: AppState = {
    ...state,
    eventLog: state.eventLog.length >= 500 ? [...state.eventLog.slice(-499), event] : [...state.eventLog, event],
  };

  switch (event.type) {
    case "TASK_STATE":
      return {
        ...next,
        task: {
          id: event.payload.taskId,
          status: event.payload.status,
          stepLabel: event.payload.stepLabel,
        },
      };
    case "MODEL_STATE":
      return { ...next, model: { modelId: event.payload.modelId, residency: event.payload.residency, verifiedAtLoad: event.payload.verifiedAtLoad } };
    case "POLICY_DECISION":
      return { ...next, policy: { actionClass: event.payload.actionClass, tier: event.payload.tier, verdict: event.payload.verdict, reason: event.payload.reason } };
    case "VERIFICATION_OUTCOME":
      return { ...next, verification: { outcome: event.payload.outcome, evidenceCount: event.payload.evidenceCount, timestamp: event.payload.timestamp } };
    case "AUTHORIZATION_REQUIRED":
      return { ...next, authorization: { tier: event.payload.tier, summary: event.payload.summary } };
    default:
      return next;
  }
}

export const initialState: AppState = {
  eventLog: [],
  task: null,
  model: null,
  policy: null,
  verification: null,
  authorization: null,
  ui: { panels: {} },
};

export function submitTask(text: string): SubmitTaskRequest {
  return { task_text: text };
}
