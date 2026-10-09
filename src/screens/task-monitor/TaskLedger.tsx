import { memo } from "react";
import { AppState, TaskStatus, AppEvent } from "../../state/app";

const STATUS_COLOR: Record<TaskStatus, string> = {
  PENDING: "text-muted",
  RUNNING: "text-primary",
  VERIFIED: "sem-verified",
  LIKELY_SUCCESS: "text-secondary",
  UNKNOWN: "text-muted",
  FAILED: "sem-violation",
  ABORTED: "sem-violation",
  PARKED: "sem-caution",
};

const EXEC_OUTCOME_LABEL: Record<string, string> = {
  true: "OK",
  false: "FAIL",
};

function statusClass(status: TaskStatus): string {
  return STATUS_COLOR[status] ?? "text-muted";
}

export const TaskLedger = memo(function TaskLedger({ state }: { state: AppState }) {
  const rows = state.eventLog;

  return (
    <div className="overflow-auto rounded-none border-1px line bg-surface-1">
      <table className="w-full text-[12px] data-mono">
        <thead className="sticky top-0 z-10 border-b-1px line-strong bg-surface-2">
          <tr>
            <th className="px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold">TIME</th>
            <th className="px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold">EPOCH</th>
            <th className="px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold">TARGET</th>
            <th className="px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold">ACTION</th>
            <th className="px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold">CLASS</th>
            <th className="px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold">TIER</th>
            <th className="px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold">POLICY</th>
            <th className="px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold">EXEC</th>
            <th className="px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold">VERIFY</th>
          </tr>
        </thead>
        <tbody>
          {rows.length === 0 ? (
            <tr>
              <td className="px-3 py-6 text-center text-muted text-[12px]">No events yet.</td>
            </tr>
          ) : (
            rows.map((event, i) => {
              const policy = event.type === "POLICY_DECISION" ? event.payload : null;
              const verification =
                event.type === "VERIFICATION_OUTCOME" ? event.payload : null;
              const execution =
                event.type === "EXECUTION_RESULT" ? event.payload : null;
              const task = state.task;

              const eventLabel = (event.type)
                .replace("_", " ")
                .toUpperCase();

              return (
                <tr key={i} className="border-b-1px line">
                  <td className="px-3 py-2 text-muted">{i + 1}</td>
                  <td className="px-3 py-2">{task?.stepLabel ?? "—"}</td>
                  <td className="px-3 py-2 text-muted">{policy?.action_class ?? execution?.action ?? "—"}</td>
                  <td className="px-3 py-2 text-primary data-mono uppercase">
                    {eventLabel}
                  </td>
                  <td className="px-3 py-2 text-muted">{policy?.action_class ?? execution?.action ?? "—"}</td>
                  <td className={`px-3 py-2 ${policy ? "text-primary data-mono" : "text-muted"}`}>
                    {policy?.tier ?? "—"}
                  </td>
                  <td className={`px-3 py-2 ${policy ? "text-primary" : "text-muted"}`}>
                    {policy?.verdict ?? "—"}
                  </td>
                  <td className={`px-3 py-2 ${execution ? "text-primary" : "text-muted"}`}>
                    {execution ? EXEC_OUTCOME_LABEL[execution.success ? "true" : "false"] ?? "—" : "—"}
                  </td>
                  <td className={`px-3 py-2 ${verification ? "text-primary" : "text-muted"}`}>
                    {verification?.outcome ?? "—"}
                  </td>
                </tr>
              );
            })
          )}
        </tbody>
      </table>
    </div>
  );
});
