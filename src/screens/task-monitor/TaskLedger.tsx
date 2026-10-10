import { memo } from "react";
import { AppState, TaskSnapshot, AppEvent } from "../../state/app";
import { useLedgerRowMotion } from "../../motion";

// Action Ledger (DESIGN.md §14.1): dense monospaced event log.
// Columns: TIME · EVENT · STEP · ACTION · TIER · POLICY · VERIFY
// Every rendered value is a literal backend payload field; the frontend
// never derives a classification it was not emitted.

// Verification outcomes render exactly as the backend emits them, with a
// restrained status tint. UNKNOWN stays neutral — it is not a hue.
function verifyCell(outcome: string): { text: string; cls: string } {
  switch (outcome) {
    case "VERIFIED_SUCCESS":
      return { text: outcome, cls: "sem-verified" };
    case "VERIFIED_FAILURE":
    case "LIKELY_FAILURE":
      return { text: outcome, cls: "sem-violation" };
    case "LIKELY_SUCCESS":
      return { text: outcome, cls: "text-primary" };
    default:
      return { text: outcome, cls: "text-secondary" };
  }
}

const TH =
  "px-3 py-1.5 text-left text-[11px] font-semibold uppercase tracking-wider-safe text-muted whitespace-nowrap";

const LedgerRow = memo(function LedgerRow({
  event,
  index,
  task,
}: {
  event: AppEvent;
  index: number;
  task: TaskSnapshot | null;
}) {
  const rowRef = useLedgerRowMotion<HTMLTableRowElement>();

  const policy = event.type === "POLICY_DECISION" ? event.payload : null;
  const verification =
    event.type === "VERIFICATION_OUTCOME" ? event.payload : null;
  const execution =
    event.type === "EXECUTION_RESULT" ? event.payload : null;
  const action = event.type === "ACTION_PROPOSAL" ? event.payload : null;

  // Backend event types are already canonical vocabulary.
  const eventLabel = event.type.replace(/_/g, " ");

  return (
    <tr ref={rowRef} className="border-b-1px line hover:bg-bg-1">
      <td className="px-3 py-1.5 text-muted whitespace-nowrap">
        {(index + 1).toString().padStart(3, "0")}
      </td>
      <td className="px-3 py-1.5 text-primary whitespace-nowrap">
        {eventLabel}
      </td>
      <td className="px-3 py-1.5 text-secondary whitespace-nowrap">
        {task ? `STEP ${task.stepLabel}` : "—"}
      </td>
      <td className="px-3 py-1.5 text-secondary data-mono whitespace-nowrap">
        {action?.action ?? execution?.action ?? policy?.action_class ?? "—"}
      </td>
      <td
        className={`px-3 py-1.5 whitespace-nowrap ${
          policy ? "text-primary" : "text-muted"
        }`}
      >
        {policy?.tier ?? "—"}
      </td>
      <td
        className={`px-3 py-1.5 whitespace-nowrap ${
          policy ? "text-primary" : "text-muted"
        }`}
      >
        {policy?.verdict ?? "—"}
      </td>
      <td className="px-3 py-1.5 whitespace-nowrap">
        {verification ? (
          <span className={verifyCell(verification.outcome).cls}>
            {verifyCell(verification.outcome).text}
          </span>
        ) : (
          <span className="text-muted">—</span>
        )}
      </td>
    </tr>
  );
});

export const TaskLedger = memo(function TaskLedger({ state }: { state: AppState }) {
  const rows = state.eventLog;
  const task = state.task;

  return (
    <div className="min-w-0 rounded-none border-1px line bg-surface-1">
      <div className="flex items-center justify-between border-b-1px line px-3 py-2">
        <span className="label-uppercase text-secondary">EVENT LOG</span>
        <span className="micro-annotation text-muted data-mono">
          {rows.length.toString().padStart(3, "0")} EVENTS · REPLAYABLE
        </span>
      </div>

      <div className="max-h-[480px] min-h-[120px] overflow-auto">
        {rows.length === 0 ? (
          <div className="border border-dashed border-line-strong m-3 px-4 py-8">
            <p className="text-[12.5px] text-secondary">
              No runtime events received.
            </p>
            <p className="micro-annotation mt-1 text-muted">
              Submit a task to begin the runtime event stream.
            </p>
          </div>
        ) : (
          <table className="w-full text-left text-[12.5px] data-mono">
            <thead className="sticky top-0 z-10 bg-surface-2">
              <tr className="border-b-1px line-strong">
                <th className={TH}>TIME</th>
                <th className={TH}>EVENT</th>
                <th className={TH}>STEP</th>
                <th className={TH}>ACTION</th>
                <th className={TH}>TIER</th>
                <th className={TH}>POLICY</th>
                <th className={TH}>VERIFY</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((event, i) => (
                <LedgerRow key={i} event={event} index={i} task={task} />
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
});
