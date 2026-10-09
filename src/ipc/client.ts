// Tauri IPC client boundary.
//
// Allowlisted commands/events only. No arbitrary command blobs.

import { invoke } from "@tauri-apps/api/core";
import type {
  SubmitTaskRequest,
  SubmitTaskResult,
  BeginHumanHandoffRequest,
  HumanHandoffResult,
} from "./schemas";

export async function submitTask(request: SubmitTaskRequest): Promise<SubmitTaskResult> {
  return invoke<SubmitTaskResult>("app_submit_task", { request });
}

export async function abortTask(taskId: string): Promise<void> {
  await invoke("app_abort_task", { request: { task_id: taskId } });
}

export async function beginHumanHandoff(
  request: BeginHumanHandoffRequest,
): Promise<HumanHandoffResult> {
  return invoke<HumanHandoffResult>("app_begin_human_handoff", { request });
}
