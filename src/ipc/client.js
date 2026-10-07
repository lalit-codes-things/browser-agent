// Tauri IPC client boundary.
//
// Allowlisted commands/events only. No arbitrary command blobs.
import { invoke } from "@tauri-apps/api/core";
export async function submitTask(request) {
    // Placeholder command; wired when Tauri command side exists.
    return invoke("app_submit_task", { request });
}
