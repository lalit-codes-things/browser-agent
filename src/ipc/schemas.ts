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

export interface EndHumanHandoffRequest {
  task_id: string;
}

export interface ConfirmAuthorizationRequest {
  task_id: string;
  commitment_hash: string;
}

export interface DenyAuthorizationRequest {
  task_id: string;
}

export interface HumanHandoffResult {
  durable_id: string;
  durable_state: ActionDurableState;
}

export type ModelAvailability =
  | "VERIFIED_LOADED"
  | "PRESENT_UNVERIFIED"
  | "UNAVAILABLE";

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

export interface NavigateRequest {
  url: string;
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
  | { type: "AUTHORIZATION_RESOLVED"; payload: AuthorizationResolvedEvent }
  | { type: "AGENT_CURSOR_STATE"; payload: AgentCursorStateEvent }
  | { type: "HUMAN_TAKEOVER_STATE"; payload: HumanTakeoverStateEvent }
  | { type: "ACTION_DURABLE_STATE"; payload: ActionDurableStateEvent }
  | { type: "MODEL_STATE"; payload: ModelStateEvent }
  | { type: "EGRESS_STATE"; payload: EgressStateEvent };

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
  /** Backend-issued commitment reference; echoed back to identify the request. */
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

export interface StoredTask {
  id: string;
  status: TaskStatus;
  progress: {
    task_id: string;
    current_state: TaskStatus;
    step_index: number;
    total_steps: number;
    epoch: number;
    mode: string;
  };
  preserved_state_summary?: string;
}

export interface VaultStatus {
  is_initialized: boolean;
  is_unlocked: boolean;
  item_count: number;
}

export interface CreateVaultRequest {
  password: string;
}

export interface UnlockVaultRequest {
  password: string;
}

export interface ChangeVaultPasswordRequest {
  old_password: string;
  new_password: string;
}

export interface VaultItemRef {
  id: string;
  origin: string;
  account_label: string;
  scope: { exactorigin?: { origin: string }; multisubdomain?: { domain: string }; sensitivetoken?: null };
  https_check_state: "PASS" | "FAIL" | "UNKNOWN";
  idn_homograph_check_state: "PASS" | "MISMATCH" | "UNKNOWN";
  last_used_task?: string;
}

export interface AddVaultItemRequest {
  origin: string;
  account_label: string;
  secret: string;
}

export interface DeleteVaultItemRequest {
  id: string;
}

export interface AuditSegment {
  segment_id: string;
  sequence: number;
  hmac_link: string;
  segment_hmac: string;
}

export type ChainVerificationStatus =
  | "Verified"
  | { Tampered: { at: number } }
  | "NotVerifiable";

export interface ChainVerificationResult {
  status: ChainVerificationStatus;
  verified_at?: string;
}

export type SkillStatus = "ACTIVE" | "SHADOW" | "SUSPENDED_DRIFT" | "REVOKED";

export interface SkillRecord {
  id: string;
  version: string;
  origin_scope: string;
  status: SkillStatus;
  commitment_hash: string;
  last_shadow_eval?: string;
}

export interface ApproveSkillVersionRequest {
  id: string;
  version: string;
}

export type DownloadVerificationState = "QUARANTINED" | "VERIFIED" | "REJECTED";

export interface QuarantinedDownload {
  id: string;
  file_path: string;
  size_bytes: number;
  declared_type?: string;
  verification_state: DownloadVerificationState;
}

export interface VerifyDownloadRequest {
  id: string;
}

export interface BrowserProfile {
  id: string;
  kind: "EPHEMERAL" | { PERSISTENT: { origin: string; expires_at?: string; storage_cap_bytes?: number } };
  storage_used_bytes: number;
}

export interface PurgeProfileRequest {
  id: string;
}

export interface Settings {
  runtime: {
    announce_latency: boolean;
  };
  browser: {
    headless_allowed: boolean;
  };
  model: {
    use_provisional_model: boolean;
  };
  network: {
    egress_mode: "ENFORCED" | "DISABLED";
    quic_blocked: boolean;
    doh_disabled: boolean;
  };
  vault: {
    master_key_storage: string;
  };
  security: {
    high_stakes_threshold_amount?: string;
  };
  audit: {
    retention_policy: string;
  };
  keyboard: {
    reduced_motion: boolean;
  };
}

export interface UpdateSettingsRequest {
  settings: Settings;
}

