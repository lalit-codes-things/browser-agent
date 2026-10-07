// Tauri IPC client boundary.
//
// Allowlisted commands/events only. No arbitrary command blobs.

import { invoke } from "@tauri-apps/api/core";
import type { SubmitTaskRequest, SubmitTaskResult } from "./schemas";

export async function submitTask(request: SubmitTaskRequest): Promise<SubmitTaskResult> {
  // Placeholder command; wired when Tauri command side exists.
  return invoke<SubmitTaskResult>("app_submit_task", { request });
}
