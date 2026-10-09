import { abortTask } from "../../ipc/client";
import { memo } from "react";
import { AppState } from "../../state/app";

import { useStatusMotion } from "../../motion";

export const StatusSpine = memo(function StatusSpine({ state }: { state: AppState }) {
  const motionRef = useStatusMotion(Boolean(state.task));

  const modelResidency = state.model?.residency ?? "UNKNOWN";
  const modelVerified = state.model?.verifiedAtLoad ? "VERIFIED" : "NOT_TESTED";

  const browser = state.browser ?? { available: false, state: "UNKNOWN" as const, reason: undefined };
  const browserState = browser.state;
  const browserAvailable = browser.available;

  const egress = "UNKNOWN";
  const highRisk = "NOT_EMITTED";
  const audit = "NOT_EMITTED";

  const taskState = state.task?.status ?? "UNKNOWN";
  const taskStep = state.task?.stepLabel ?? "UNKNOWN";

  return (
    <header className="border-b-1px line-strong border-b bg-surface-1">
      <div ref={motionRef} className="mx-auto flex max-w-7xl items-center gap-6 px-6 py-2 text-secondary data-mono label-uppercase">
        <div className="flex items-center gap-2">
          <span className="text-primary text-[11px] font-semibold uppercase tracking-wider-safe">AGENT</span>
          <span className="text-muted text-[11px]">BROWSER AGENT</span>
        </div>

        <div className="flex items-center gap-6">
          <div className="flex items-center gap-2 text-[11px]">
            <span className="text-muted">BROWSER</span>
            <span className={"text-primary fixed-width-amount " + (browserAvailable ? "" : "text-sem-violation")}>
              {browserState}
            </span>
            {browser.reason ? (
              <span className="text-muted text-[11px]">· {browser.reason}</span>
            ) : null}
          </div>

          <div className="flex items-center gap-2 text-[11px]">
            <span className="text-muted">EGRESS</span>
            <span className="text-primary fixed-width-amount">{egress}</span>
          </div>

          <div className="flex items-center gap-2 text-[11px]">
            <span className="text-muted">MODEL</span>
            <span className="text-primary fixed-width-amount">
              {state.model?.modelId ?? "UNKNOWN"} · {modelVerified}
            </span>
            <span className="text-muted">{modelResidency}</span>
          </div>

          <div className="flex items-center gap-2 text-[11px]">
            <span className="text-muted">HIGH-RISK</span>
            <span className="text-primary fixed-width-amount">{highRisk}</span>
          </div>

          <div className="flex items-center gap-2 text-[11px]">
            <span className="text-muted">AUDIT</span>
            <span className="text-primary fixed-width-amount">{audit}</span>
          </div>

          <div className="flex items-center gap-2 text-[11px]">
            <span className="text-muted">TASK</span>
            <span className="text-primary fixed-width-amount">{taskState} · STEP {taskStep}</span>
          </div>
        </div>

        <div className="ml-auto flex items-center gap-3">
          <button
            type="button"
            onClick={() => state.task && void abortTask(state.task.id)}
            disabled={!state.task}
            className="text-muted text-[11px]"
          >
            ABORT
          </button>
        </div>
      </div>
    </header>
  );
});
