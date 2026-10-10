// Tauri IPC client boundary.
//
// Allowlisted commands/events only. No arbitrary command blobs.

import { invoke } from "@tauri-apps/api/core";
import type {
  SubmitTaskRequest,
  SubmitTaskResult,
  BeginHumanHandoffRequest,
  HumanHandoffResult,
  EndHumanHandoffRequest,
  ConfirmAuthorizationRequest,
  DenyAuthorizationRequest,
  ModelStateEvent,
  EgressStateEvent,
  NavigateRequest,
  StoredTask,
  VaultStatus,
  VaultItemRef,
  ChainVerificationResult,
  AuditSegment,
  SkillRecord,
  QuarantinedDownload,
  BrowserProfile,
  Settings,
  CreateVaultRequest,
  UnlockVaultRequest,
  ChangeVaultPasswordRequest,
  AddVaultItemRequest,
  DeleteVaultItemRequest,
  ApproveSkillVersionRequest,
  VerifyDownloadRequest,
  PurgeProfileRequest,
  UpdateSettingsRequest,
} from "./schemas";

export async function submitTask(request: SubmitTaskRequest): Promise<SubmitTaskResult> {
  return invoke<SubmitTaskResult>("app_submit_task", { request });
}

export async function abortTask(taskId: string): Promise<void> {
  await invoke("app_abort_task", { request: { task_id: taskId } });
}

export async function listTasks(): Promise<StoredTask[]> {
  return invoke<StoredTask[]>("app_list_tasks");
}

export async function beginHumanHandoff(
  request: BeginHumanHandoffRequest,
): Promise<HumanHandoffResult> {
  return invoke<HumanHandoffResult>("app_begin_human_handoff", { request });
}

export async function endHumanHandoff(request: EndHumanHandoffRequest): Promise<void> {
  await invoke("app_end_human_handoff", { request });
}

/** Confirm a backend-issued authorization request by echoing its commitment hash. */
export async function confirmAuthorization(
  request: ConfirmAuthorizationRequest,
): Promise<void> {
  await invoke("app_confirm_authorization", { request });
}

/** Deny a backend-issued authorization request; the backend stops the task. */
export async function denyAuthorization(request: DenyAuthorizationRequest): Promise<void> {
  await invoke("app_deny_authorization", { request });
}

export async function getModelStatus(): Promise<ModelStateEvent> {
  return invoke<ModelStateEvent>("app_model_status");
}

export async function getEgressStatus(): Promise<EgressStateEvent> {
  return invoke<EgressStateEvent>("app_egress_status");
}

export async function navigateTo(request: NavigateRequest): Promise<void> {
  await invoke("app_navigate", { request });
}

export async function goBack(): Promise<void> {
  await invoke("app_back");
}

export async function goForward(): Promise<void> {
  await invoke("app_forward");
}

export async function reloadPage(): Promise<void> {
  await invoke("app_reload");
}

// --- Vault Operations ---

export async function getVaultStatus(): Promise<VaultStatus> {
  return invoke<VaultStatus>("app_vault_status");
}

export async function createVault(request: CreateVaultRequest): Promise<void> {
  await invoke("app_vault_create", { request });
}

export async function unlockVault(request: UnlockVaultRequest): Promise<void> {
  await invoke("app_vault_unlock", { request });
}

export async function lockVault(): Promise<void> {
  await invoke("app_vault_lock");
}

export async function changeVaultPassword(
  request: ChangeVaultPasswordRequest,
): Promise<void> {
  await invoke("app_vault_change_password", { request });
}

export async function listVaultItems(): Promise<VaultItemRef[]> {
  return invoke<VaultItemRef[]>("app_vault_list_items");
}

export async function addVaultItem(request: AddVaultItemRequest): Promise<VaultItemRef> {
  return invoke<VaultItemRef>("app_vault_add_item", { request });
}

export async function deleteVaultItem(request: DeleteVaultItemRequest): Promise<boolean> {
  return invoke<boolean>("app_vault_delete_item", { request });
}

// --- Audit Operations ---

export async function verifyAuditChain(): Promise<ChainVerificationResult> {
  return invoke<ChainVerificationResult>("app_audit_verify_chain");
}

export async function getAuditRecords(): Promise<AuditSegment[]> {
  return invoke<AuditSegment[]>("app_audit_records");
}

// --- Skills Operations ---

export async function listSkills(): Promise<SkillRecord[]> {
  return invoke<SkillRecord[]>("app_skills_list");
}

export async function approveSkillVersion(request: ApproveSkillVersionRequest): Promise<void> {
  await invoke("app_skill_approve_version", { request });
}

// --- Quarantined Downloads ---

export async function listQuarantinedDownloads(): Promise<QuarantinedDownload[]> {
  return invoke<QuarantinedDownload[]>("app_quarantine_list");
}

export async function verifyQuarantinedDownload(request: VerifyDownloadRequest): Promise<void> {
  await invoke("app_quarantine_verify", { request });
}

// --- Profiles ---

export async function listBrowserProfiles(): Promise<BrowserProfile[]> {
  return invoke<BrowserProfile[]>("app_profiles_list");
}

export async function purgeProfile(request: PurgeProfileRequest): Promise<void> {
  await invoke("app_profiles_purge", { request });
}

// --- Settings ---

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>("app_get_settings");
}

export async function updateSettings(request: UpdateSettingsRequest): Promise<void> {
  await invoke("app_update_settings", { request });
}
