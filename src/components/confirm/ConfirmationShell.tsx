import { useCallback, memo } from "react";
import { AppState } from "../../state/app";
import { confirmAuthorization, denyAuthorization } from "../../ipc/client";
import type { HumanTakeoverMode } from "../../ipc/schemas";

const HANDOFF_MODE_LABEL: Record<HumanTakeoverMode, string> = {
  IDLE: "IDLE",
  HANDOFF_REQUESTED: "HANDOFF REQUESTED",
  HANDOFF_ACTIVE: "HANDOFF ACTIVE",
  HANDOFF_DENIED: "HANDOFF DENIED",
  HANDOFF_EXPIRED: "HANDOFF EXPIRED",
};

// Authorization interlock (DESIGN.md §15): the most important visual surface.
// Flat dark scrim, no blur, hard 1px rule, zero-to-near-zero radius, stable
// from first frame. Security content never animates. Escape means deny;
// clicking the scrim does nothing.

export const ConfirmationShell = memo(function ConfirmationShell({ state }: { state: AppState }) {
  const auth = state.authorization;
  const open = !!auth;
  const highStakes = auth?.tier === "BIOMETRIC_CONFIRM";

  // The frontend only echoes the backend-issued request identity; the
  // backend revalidates the commitment, epoch, and scope before acting.
  const handleConfirm = useCallback(() => {
    if (!auth) return;
    void confirmAuthorization({
      task_id: auth.taskId,
      commitment_hash: auth.commitmentHash,
    }).catch(() => undefined);
  }, [auth]);

  const handleDeny = useCallback(() => {
    if (!auth) return;
    void denyAuthorization({ task_id: auth.taskId }).catch(() => undefined);
  }, [auth]);

  if (!open) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="auth-heading"
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          handleDeny();
        }
      }}
      className="fixed inset-0 z-50 flex items-center justify-center bg-[#0A0B0C]"
    >
      {/* Scrim carries no click behavior — escape means deny (§15). */}
      <div className="pointer-events-none absolute inset-0" aria-hidden="true" />

      <div
        className="
          relative flex max-h-[90vh] w-[min(560px,90vw)] flex-col
          border-1px line-strong bg-surface-1
          max-radius-2 shadow-none
        "
      >
        {/* Tier header band — HIGH-STAKES is structurally more prominent:
            a full-width caution-tinted band with a double rule, not merely
            recolored text (§15). */}
        <div
          className={`
            mb-5 grid grid-cols-[8px_1fr] items-stretch
            ${highStakes ? "bg-surface-2" : ""}
          `}
        >
          <div className={highStakes ? "sem-caution-bg" : "bg-line"} aria-hidden="true" />
          <div className="px-4 py-3">
            <h2
              id="auth-heading"
              className={`
                label-uppercase
                ${highStakes
                  ? "text-[13px] tracking-[0.12em] sem-caution"
                  : "text-[11px] text-primary"}
              `}
            >
              {highStakes ? "HIGH-STAKES AUTHORIZATION" : "AUTHORIZATION REQUIRED"}
            </h2>
            {highStakes ? (
              <p className="micro-annotation mt-1 sem-caution">
                Fresh biometric confirmation required.
              </p>
            ) : null}
          </div>
        </div>

        <div className="overflow-auto px-5 pb-5">
          <dl className="mb-5 max-w-md space-y-2 text-[12.5px]">
            <div className="flex gap-4">
              <dt className="label-uppercase w-24 shrink-0 pt-0.5 text-muted">TIER</dt>
              <dd className="data-mono fixed-width-amount text-primary">{auth?.tier ?? "UNKNOWN"}</dd>
            </div>
            <div className="flex gap-4">
              <dt className="label-uppercase w-24 shrink-0 pt-0.5 text-muted">SUMMARY</dt>
              <dd className="data-mono text-primary">{auth?.summary ?? "UNKNOWN"}</dd>
            </div>
            <div className="flex gap-4">
              <dt className="label-uppercase w-24 shrink-0 pt-0.5 text-muted">COMMITMENT</dt>
              <dd className="data-mono fixed-width-amount break-all text-primary">
                {auth?.commitmentHash ?? "UNKNOWN"}
              </dd>
            </div>
          </dl>

          <p className="mb-6 text-[12.5px] text-secondary">
            Trusted Policy requires approval before execution.
          </p>            <div className="flex flex-wrap gap-3">
            <button
              type="button"
              onClick={handleDeny}
              className="
                min-w-[200px] border-1px line bg-surface-2 px-4 py-2
                text-[12.5px] text-primary uppercase tracking-wider-safe
                hover:bg-surface-3 active:bg-bg-3
                transition-colors duration-120 max-radius-2
              "
            >
              DENY AND STOP TASK
            </button>
            <button
              type="button"
              onClick={handleConfirm}
              className="
                min-w-[200px] border-1px line-strong bg-surface-3 px-4 py-2
                text-[12.5px] font-semibold text-primary uppercase tracking-wider-safe
                hover:bg-bg-3 active:bg-surface-2
                transition-colors duration-120 max-radius-2
              "
            >
              APPROVE AUTHORIZATION
            </button>
          </div>

          {state.humanTakeover && state.humanTakeover.mode !== "IDLE" ? (
            <div className="mt-5 border-t-1px line pt-4">
              <div className="mb-1 flex items-center justify-between">
                <span className="label-uppercase text-muted">HUMAN HANDOFF</span>
                <span className="text-[11px] data-mono uppercase text-primary">
                  {HANDOFF_MODE_LABEL[state.humanTakeover.mode] ?? state.humanTakeover.mode}
                </span>
              </div>
              {state.humanTakeover.reason ? (
                <p className="micro-annotation text-muted">{state.humanTakeover.reason}</p>
              ) : null}
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
});
