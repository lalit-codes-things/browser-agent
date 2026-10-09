// Typed IPC schemas.
//
// These mirror the Rust event enums in src-tauri/src/ipc/events.rs.
// Frontend never invents these values; they come from the backend.

export type TaskStatus =
  | "PENDING"
  | "RUNNING"
  | "VERIFIED"
  | "LIKELY_SUCCESS"
  | "UNKNOWN"
  | "FAILED"
  | "ABORTED"
  | "PARKED";

export type ModelResidency = "LOADED" | "PARKED" | "UNLOADED";

export type VerificationOutcome =
  | "VERIFIED_SUCCESS"
  | "LIKELY_SUCCESS"
  | "UNKNOWN"
  | "LIKELY_FAILURE"
  | "VERIFIED_FAILURE";

export type BrowserRuntimeState =
  | "READY"
  | "UNAVAILABLE"
  | "CRASHED"
  | "STOPPED";

export type AgentCursorState =
  | "IDLE"
  | "MOVING"
  | "CLICKING"
  | "BLOCKED"
  | "HANDING_OFF";

export type HumanTakeoverMode =
  | "IDLE"
  | "HANDOFF_REQUESTED"
  | "HANDOFF_ACTIVE"
  | "HANDOFF_DENIED"
  | "HANDOFF_EXPIRED";

export type ActionDurableState =
  | "UNKNOWN"
  | "SUBMITTED"
  | "WAITING_FOR_EXTERNAL_AUTH"
  | "PROCESSING"
  | "READY"
  | "RECONCILED"
  | "RESOLVED";

export interface SubmitTaskRequest {
  task_text: string;
}

export interface SubmitTaskResult {
  task_id: string;
}

export interface BeginHumanHandoffRequest {
  task_id: string;
}

export interface HumanHandoffResult {
  durable_id: string;
  durable_state: ActionDurableState;
}

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
  | { type: "ACTION_DURABLE_STATE"; payload: ActionDurableStateEvent };;

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
