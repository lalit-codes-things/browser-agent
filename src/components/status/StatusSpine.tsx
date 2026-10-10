import { abortTask, beginHumanHandoff, endHumanHandoff } from "../../ipc/client";
import { useCallback, memo } from "react";
import {
  AppState,
  BrowserRuntimeState,
  ModelAvailability,
  TaskStatus,
} from "../../state/app";

import { useStatusMotion } from "../../motion";

// Status spine (DESIGN.md §10): permanently visible, every field mapped to
// runtime state. Fields the backend has not emitted yet render as
// NOT_EMITTED — they are never fabricated.

const BROWSER_STATE_TINT: Record<BrowserRuntimeState, string> = {
  READY: "sem-verified",
  UNAVAILABLE: "text-secondary",
  CRASHED: "sem-violation",
  STOPPED: "text-secondary",
};

const MODEL_AVAILABILITY_TINT: Record<ModelAvailability, string> = {
  VERIFIED_LOADED: "sem-verified",
  PRESENT_UNVERIFIED: "text-primary",
  UNAVAILABLE: "text-secondary",
};

const TASK_STATE_TINT: Record<TaskStatus, string> = {
  PENDING: "text-secondary",
  RUNNING: "text-primary",
  VERIFIED: "sem-verified",
  LIKELY_SUCCESS: "text-primary",
  UNKNOWN: "text-secondary",
  FAILED: "sem-violation",
  ABORTED: "sem-violation",
  PARKED: "sem-caution",
};

function SpineField({
  label,
  value,
  tint,
}: {
  label: string;
  value: string;
  tint: string;
}) {
  return (
    <div className="flex shrink-0 items-center gap-2 text-[11px]">
      <span className="text-muted">{label}</span>
      <span className={`data-mono fixed-width-amount ${tint}`}>{value}</span>
    </div>
  );
}

const SPINE_BUTTON = `
  border-1px line px-3 py-1 text-[11px] text-primary uppercase tracking-wider-safe
  hover:bg-surface-3 active:bg-bg-3 disabled:cursor-not-allowed disabled:text-muted
  transition-colors duration-120 max-radius-2
`;

export const StatusSpine = memo(function StatusSpine({ state }: { state: AppState }) {
  const motionRef = useStatusMotion(Boolean(state.task));

  const model = state.model;
  const egress = state.egress;
  const browser = state.browser;
  const task = state.task;
  const handoffActive = state.humanTakeover?.mode === "HANDOFF_ACTIVE";

  // Takeover is a real backend-owned control-mode transition (§14): the
  // agent pauses autonomous execution and the user drives the managed
  // browser; returning control advances the epoch and re-perceives.
  const handleTakeover = useCallback(() => {
    if (!task) return;
    void beginHumanHandoff({ task_id: task.id }).catch(() => undefined);
  }, [task]);

  const handleReturnControl = useCallback(() => {
    if (!task) return;
    void endHumanHandoff({ task_id: task.id }).catch(() => undefined);
  }, [task]);

  return (
    <header className="border-b-1px line-strong border-b bg-surface-1">
      <div ref={motionRef} className="flex items-center gap-5 border-b-1px line px-4 py-2 text-secondary">
        <span className="text-[11px] font-semibold uppercase tracking-wider-safe text-primary">
          AGENT
        </span>
        <span className="text-[11px] text-muted">BROWSER AGENT</span>

        <div className="ml-auto flex items-center gap-3">
          <button
            type="button"
            onClick={handleTakeover}
            disabled={!task || handoffActive}
            className={SPINE_BUTTON}
          >
            TAKEOVER
          </button>
          <button
            type="button"
            onClick={handleReturnControl}
            disabled={!task || !handoffActive}
            className={SPINE_BUTTON}
          >
            RETURN CONTROL
          </button>
          <button
            type="button"
            onClick={() => task && void abortTask(task.id)}
            disabled={!task}
            className={SPINE_BUTTON}
          >
            ABORT
          </button>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-x-5 gap-y-1 px-4 py-2 text-[11px]">
        <SpineField
          label="BROWSER"
          value={browser ? browser.state : "NOT_EMITTED"}
          tint={browser ? BROWSER_STATE_TINT[browser.state] : "text-muted"}
        />

        <SpineField
          label="EGRESS"
          value={
            egress
              ? egress.enforcementActive
                ? egress.mode
                : `${egress.mode} · NOT ACTIVE`
              : "NOT_EMITTED"
          }
          tint={
            egress
              ? egress.enforcementActive
                ? "sem-verified"
                : "text-secondary"
              : "text-muted"
          }
        />

        <SpineField
          label="MODEL"
          value={
            model
              ? `${model.modelId} · ${model.residency} · ${model.availability}`
              : "NOT_EMITTED"
          }
          tint={model ? MODEL_AVAILABILITY_TINT[model.availability] : "text-muted"}
        />

        <SpineField
          label="HIGH-RISK"
          value="NOT_EMITTED"
          tint="text-muted"
        />

        <SpineField
          label="AUDIT"
          value="NOT_EMITTED"
          tint="text-muted"
        />

        <SpineField
          label="TASK"
          value={
            task
              ? `${task.status} · STEP ${task.stepLabel}`
              : "NOT_EMITTED"
          }
          tint={task ? TASK_STATE_TINT[task.status] : "text-muted"}
        />
      </div>
    </header>
  );
});
