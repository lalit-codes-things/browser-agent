import { memo } from "react";
import { AppState } from "../../state/app";

export const StatusSpine = memo(function StatusSpine({ state }: { state: AppState }) {
  const egress =
    state.policy?.verdict === "DENIED" ? "ENFORCED" : "UNKNOWN";

  const modelResidency = state.model?.residency ?? "UNKNOWN";
  const modelVerified = state.model?.verifiedAtLoad ? "VERIFIED" : "NOT_TESTED";

  const highRisk = state.authorization?.tier === "BIOMETRIC_CONFIRM" ? "1/1" : "0/1";

  const audit = state.verification?.outcome === "VERIFIED_SUCCESS"
    ? `CHAIN VERIFIED ${state.verification.timestamp ?? ""}`.trim()
    : "UNKNOWN";

  const taskState = state.task?.status ?? "UNKNOWN";
  const taskStep = state.task?.stepLabel ?? "0 / 0";

  return (
    <header className="border-b-1px line-strong border-b bg-surface-1">
      <div className="mx-auto flex max-w-7xl items-center gap-6 px-6 py-2 text-secondary data-mono label-uppercase">
        <div className="flex items-center gap-2">
          <span className="text-primary text-[11px] font-semibold uppercase tracking-wider-safe">AGENT</span>
          <span className="text-muted text-[11px]">BROWSER AGENT</span>
        </div>

        <div className="flex items-center gap-6">
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
          <span className="text-muted text-[11px]">ABORT</span>
        </div>
      </div>
    </header>
  );
});
