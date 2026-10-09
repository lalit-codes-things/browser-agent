import { invoke } from "@tauri-apps/api/core";
import { useCallback, memo } from "react";
import { AppState } from "../../state/app";
import { beginHumanHandoff } from "../../ipc/client";
import type { HumanTakeoverMode } from "../../ipc/schemas";

const HANDOFF_MODE_LABEL: Record<HumanTakeoverMode, string> = {
  IDLE: "IDLE",
  HANDOFF_REQUESTED: "HANDOFF REQUESTED",
  HANDOFF_ACTIVE: "HANDOFF ACTIVE",
  HANDOFF_DENIED: "HANDOFF DENIED",
  HANDOFF_EXPIRED: "HANDOFF EXPIRED",
};

// Authorization interlock is the most important visual surface.
// It is not a generic component-library modal. It uses hard borders,
// near-zero radius, flat dark scrim, no blur, and distinct geometry.
// (DESIGN.md #15)

export const ConfirmationShell = memo(function ConfirmationShell({ state }: { state: AppState }) {
  const auth = state.authorization;
  const open = !!auth;

  const handleConfirm = useCallback(() => {
    void invoke("app_confirm_authorization").catch(() => undefined);
  }, []);

  const handleDeny = useCallback(() => {
    void invoke("app_deny_authorization").catch(() => undefined);
  }, []);

  // Human handoff is now part of the authoritative IPC surface.
  const handleHandoff = useCallback(() => {
    if (!auth?.taskId) return;
    void beginHumanHandoff({ task_id: auth.taskId }).catch(() => undefined);
  }, [auth]);

  if (!open) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="auth-heading"
      className="fixed inset-0 z-50 flex items-center justify-center bg-[#0a0b0c]"
    >
      {/* scrim does nothing on click — no close on overlay */}
      <div className="pointer-events-none h-full w-full" aria-hidden="true" />

      <div
        className="
          relative max-w-lg overflow-hidden border-1px line-strong
          bg-surface-1 p-6 text-left shadow-none
          max-radius-2
        "
      >
        <h2
          id="auth-heading"
          className="mb-4 text-primary label-uppercase"
        >
          {auth?.tier === "BIOMETRIC_CONFIRM"
            ? "HIGH-STAKES AUTHORIZATION"
            : "AUTHORIZATION REQUIRED"}
        </h2>

        <dl className="mb-6 grid grid-cols-1 gap-3 text-[12.5px]">
          <div className="flex flex-wrap gap-x-6 gap-y-1">
            <dt className="text-muted w-28 shrink-0">TIER</dt>
            <dd className="text-primary data-mono fixed-width-amount">{auth?.tier ?? "UNKNOWN"}</dd>
          </div>
          <div className="flex flex-wrap gap-x-6 gap-y-1">
            <dt className="text-muted w-28 shrink-0">SUMMARY</dt>
            <dd className="text-primary data-mono">{auth?.summary ?? "UNKNOWN"}</dd>
          </div>
        </dl>

        <p className="mb-6 text-muted body-text">
          Trusted Policy requires approval before execution.
        </p>

        <div className="flex flex-wrap gap-3">
          <button
            type="button"
            onClick={handleDeny}
            className="
              min-w-[180px] border-1px line-strong bg-surface-2
              px-4 py-2 text-primary body-text
              hover:bg-surface-3 active:bg-bg-3
              transition-colors duration-120
              max-radius-2
            "
          >
            DENY AND STOP TASK
          </button>
          <button
            type="button"
            onClick={handleConfirm}
            className="
              min-w-[180px] border-1px line-strong
              bg-surface-2 text-primary body-text
              hover:bg-surface-3 active:bg-bg-3
              transition-colors duration-120
              max-radius-2
            "
          >
            APPROVE AUTHORIZATION
          </button>
        </div>

        {state.humanTakeover ? (
          <div className="mt-5 border-t-1px line pt-4">
            <div className="mb-2 flex items-center justify-between">
              <span className="text-muted text-[11px] uppercase tracking-wider-safe">HUMAN HANDOFF</span>
              <span className="text-primary text-[11px] data-mono uppercase">
                {HANDOFF_MODE_LABEL[state.humanTakeover.mode] ?? state.humanTakeover.mode}
              </span>
            </div>
            {state.humanTakeover.reason ? (
              <p className="text-muted text-[11px]">{state.humanTakeover.reason}</p>
            ) : null}
          </div>
        ) : null}
      </div>
    </div>
  );
});

