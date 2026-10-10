import { memo, useCallback, useState } from "react";
import type { AppState } from "../../state/app";
import { beginHumanHandoff, endHumanHandoff } from "../../ipc/client";
import { formatIpcError } from "../../ipc/errors";

interface HumanTakeoverInterlockProps {
  state: AppState;
}

export const HumanTakeoverInterlock = memo(function HumanTakeoverInterlock({
  state,
}: HumanTakeoverInterlockProps) {
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const takeover = state.humanTakeover;
  const taskId = state.task?.id;
  const isActive =
    takeover?.mode === "HANDOFF_ACTIVE" || takeover?.mode === "HANDOFF_REQUESTED";

  const handleReturnControl = useCallback(async () => {
    if (!taskId) return;
    setError(null);
    setSubmitting(true);
    try {
      await endHumanHandoff({ task_id: taskId });
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setSubmitting(false);
    }
  }, [taskId]);

  const handleTakeover = useCallback(async () => {
    if (!taskId) return;
    setError(null);
    setSubmitting(true);
    try {
      await beginHumanHandoff({ task_id: taskId });
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setSubmitting(false);
    }
  }, [taskId]);

  if (!taskId) {
    return null;
  }

  if (isActive) {
    return (
      <div className="border-b-1px sem-caution bg-bg-2 px-6 py-3">
        <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex flex-col gap-0.5">
            <div className="flex items-center gap-2">
              <span className="bg-sem-caution px-1.5 py-0.5 text-[10px] font-bold text-bg-0 uppercase tracking-wider-safe">
                HUMAN TAKEOVER ACTIVE
              </span>
              <span className="text-[12px] font-semibold text-primary">
                Autonomous agent execution is paused
              </span>
            </div>
            <p className="text-[11px] text-secondary">
              {takeover?.reason ??
                "You have full control of the managed browser. Complete your action, then return control."}
            </p>
          </div>

          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={() => void handleReturnControl()}
              disabled={submitting}
              className="
                border-1px sem-caution bg-surface-3 px-4 py-1.5
                text-[11px] font-semibold text-primary uppercase tracking-wider-safe
                hover:bg-bg-3 active:bg-surface-2 disabled:cursor-not-allowed disabled:text-muted
                transition-colors duration-120 max-radius-2
              "
            >
              {submitting ? "ADVANCING EPOCH..." : "RETURN CONTROL TO AGENT"}
            </button>
          </div>
        </div>

        {error ? (
          <p className="mt-1.5 text-[11px] data-mono text-sem-violation">{error}</p>
        ) : null}
      </div>
    );
  }

  return (
    <div className="border-b-1px line bg-surface-1 px-6 py-1.5">
      <div className="flex items-center justify-between">
        <span className="micro-annotation text-muted">
          AUTONOMOUS SUPERVISION ACTIVE · EPOCH {state.perception?.epoch ?? 1}
        </span>
        <button
          type="button"
          onClick={() => void handleTakeover()}
          disabled={submitting}
          className="
            border-1px line bg-surface-2 px-3 py-1
            text-[10px] font-semibold text-secondary uppercase tracking-wider-safe
            hover:bg-surface-3 hover:text-primary active:bg-bg-3
            disabled:cursor-not-allowed disabled:text-muted transition-colors duration-120 max-radius-2
          "
        >
          {submitting ? "PAUSING..." : "TAKE OVER BROWSER"}
        </button>
      </div>
      {error ? (
        <p className="mt-1 text-[11px] data-mono text-sem-violation">{error}</p>
      ) : null}
    </div>
  );
});
