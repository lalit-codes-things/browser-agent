// Policy Screen — real-time Policy decision log.
//
// Every value originates from backend POLICY_DECISION events.
// The frontend does not invent verdicts, tiers, or action classes.
// DESIGN.md §7: operations, table/list format, monospace fields.

import { memo } from "react";
import type { AppState } from "../../state/app";

interface PolicyDecisionRow {
  taskId: string;
  actionClass: string;
  tier: string;
  verdict: string;
  reason: string;
}

// Verdict vocabulary is the backend's exact wire values (C-06/C-07).
function verdictTint(verdict: string): string {
  if (verdict === "ALLOW") return "text-primary";
  if (verdict === "DENY") return "sem-violation";
  if (verdict === "CONFIRMATION_REQUIRED") return "sem-caution";
  return "text-secondary";
}

function tierTint(tier: string): string {
  if (tier === "BIOMETRIC_CONFIRM") return "sem-caution";
  if (tier === "NATIVE_CONFIRM") return "text-primary";
  if (tier === "NONE" || tier === "POLICY_ONLY") return "text-secondary";
  return "text-muted";
}

const PolicyRow = memo(function PolicyRow({ row }: { row: PolicyDecisionRow }) {
  return (
    <tr className="border-b-1px line hover:bg-surface-2 transition-colors duration-120">
      <td className="px-3 py-2 data-mono text-[11px] text-secondary">
        {row.taskId.slice(0, 12)}...
      </td>
      <td className="px-3 py-2 data-mono text-[11px] text-primary">{row.actionClass}</td>
      <td className={`px-3 py-2 data-mono text-[11px] font-semibold ${tierTint(row.tier)}`}>
        {row.tier}
      </td>
      <td className={`px-3 py-2 data-mono text-[11px] font-semibold ${verdictTint(row.verdict)}`}>
        {row.verdict}
      </td>
      <td className="px-3 py-2 text-[11px] text-secondary max-w-xs truncate">{row.reason}</td>
    </tr>
  );
});

export const PolicyScreen = memo(function PolicyScreen({ state }: { state: AppState }) {
  // Collect all POLICY_DECISION events from the event log. Payloads use
  // the backend wire format (snake_case); map without unsafe casts.
  const decisions: PolicyDecisionRow[] = state.eventLog
    .flatMap((e) =>
      e.type === "POLICY_DECISION"
        ? [
            {
              taskId: e.payload.task_id,
              actionClass: e.payload.action_class,
              tier: e.payload.tier,
              verdict: e.payload.verdict,
              reason: e.payload.reason,
            },
          ]
        : [],
    )
    .reverse(); // Most recent first.

  const current = state.policy;

  return (
    <main className="flex min-h-0 flex-1 flex-col">
      <div className="border-b-1px line px-6 py-4">
        <div className="flex items-baseline justify-between">
          <h1 className="section-header text-primary">POLICY ENGINE</h1>
          <span className="micro-annotation text-muted">BACKEND-AUTHORITATIVE · FRONTEND READ-ONLY</span>
        </div>
      </div>

      {/* Current decision panel */}
      {current ? (
        <section className="border-b-1px line bg-surface-1 px-6 py-4">
          <h2 className="mb-3 label-uppercase text-muted">CURRENT DECISION</h2>
          <div className="grid grid-cols-2 gap-x-8 gap-y-2 max-w-2xl text-[12px]">
            <div className="flex items-center gap-2">
              <span className="label-uppercase text-muted">TASK</span>
              <span className="data-mono text-secondary">{current.taskId.slice(0, 16)}...</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="label-uppercase text-muted">CLASS</span>
              <span className="data-mono text-primary">{current.actionClass}</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="label-uppercase text-muted">TIER</span>
              <span className={`data-mono font-semibold ${tierTint(current.tier)}`}>
                {current.tier}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <span className="label-uppercase text-muted">VERDICT</span>
              <span className={`data-mono font-semibold ${verdictTint(current.verdict)}`}>
                {current.verdict}
              </span>
            </div>
            <div className="col-span-2 flex items-start gap-2">
              <span className="label-uppercase text-muted shrink-0">REASON</span>
              <span className="text-secondary">{current.reason}</span>
            </div>
          </div>
        </section>
      ) : (
        <section className="border-b-1px line bg-surface-1 px-6 py-4">
          <p className="text-[12px] data-mono text-muted">NO ACTIVE DECISION · NOT_EMITTED</p>
        </section>
      )}

      {/* Decision log */}
      <div className="flex-1 overflow-auto p-6">
        <h2 className="mb-3 section-header text-primary">DECISION LOG</h2>
        {decisions.length === 0 ? (
          <div className="flex h-32 items-center justify-center border-1px line">
            <p className="text-[12px] data-mono text-muted">
              POLICY LOG EMPTY · DECISIONS APPEAR AFTER TASK EXECUTION
            </p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full border-collapse border-1px line text-left">
              <thead>
                <tr className="border-b-1px line bg-surface-2">
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">TASK</th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">CLASS</th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">TIER</th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">VERDICT</th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">REASON</th>
                </tr>
              </thead>
              <tbody>
                {decisions.map((row, i) => (
                  <PolicyRow key={i} row={row} />
                ))}
              </tbody>
            </table>
          </div>
        )}
        <p className="mt-4 micro-annotation text-muted">
          DECISIONS ARE BACKEND-ISSUED · FRONTEND CANNOT OVERRIDE · POLICY IS AUTHORITATIVE
        </p>
      </div>
    </main>
  );
});
