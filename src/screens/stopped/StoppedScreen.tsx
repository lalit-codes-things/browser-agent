// Stopped Screen — terminal task state surface.
//
// Rendered when task status is ABORTED or FAILED, or browser has CRASHED.
// Provides hard-stop diagnostic and safe recovery options.
// DESIGN.md §8, §13: safe stop surface, explicit outcome.

import { memo, useCallback, useState } from "react";
import type { AppState, AppEvent } from "../../state/app";
import { formatIpcError } from "../../ipc/errors";
import { abortTask, listTasks } from "../../ipc/client";

interface StoppedScreenProps {
  state: AppState;
  dispatch: React.Dispatch<AppEvent>;
}

export const StoppedScreen = memo(function StoppedScreen({ state, dispatch }: StoppedScreenProps) {
  const [dismissing, setDismissing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const task = state.task;
  const browser = state.browser;

  const isCrashed = browser?.state === "CRASHED";
  const isAborted = task?.status === "ABORTED";
  const isFailed = task?.status === "FAILED";

  // Dismiss: navigate back to console to start a new task.
  const handleDismiss = useCallback(() => {
    dispatch({ type: "SET_SCREEN", payload: "CONSOLE" });
  }, [dispatch]);

  // Hard-abort: explicit abort if somehow still running.
  const handleAbort = useCallback(async () => {
    if (!task) return;
    setError(null);
    setDismissing(true);
    try {
      await abortTask(task.id);
      dispatch({ type: "SET_SCREEN", payload: "CONSOLE" });
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setDismissing(false);
    }
  }, [task, dispatch]);

  // Refresh task history from the actual backend store.
  const handleRefreshHistory = useCallback(async () => {
    try {
      await listTasks();
    } catch {
      // Non-fatal; list shown in console when navigated there.
    }
    dispatch({ type: "SET_SCREEN", payload: "CONSOLE" });
  }, [dispatch]);

  const primaryReason =
    isCrashed
      ? "BROWSER RUNTIME CRASHED"
      : isAborted
        ? "TASK ABORTED BY USER"
        : isFailed
          ? "TASK FAILED — TERMINAL STATE"
          : "TASK STOPPED";

  const secondary =
    isCrashed
      ? browser?.reason ?? "The managed browser process exited unexpectedly."
      : task?.stepLabel ?? "No additional detail from backend.";

  return (
    <main className="flex min-h-0 flex-1 flex-col">
      <div className="border-b-1px line px-6 py-4">
        <div className="flex items-baseline justify-between">
          <h1 className="section-header sem-violation">{primaryReason}</h1>
          <span className="micro-annotation text-muted">SAFE STATE · AUDIT PRESERVED</span>
        </div>
      </div>

      <div className="flex-1 p-6">
        <div className="max-w-2xl space-y-6">
          {/* Terminal state summary */}
          <section className="border-1px border-line bg-surface-1 p-5">
            <div className="space-y-3">
              <div className="flex items-start gap-3">
                <span className="label-uppercase text-muted shrink-0 pt-0.5">STATUS</span>
                <span className={`data-mono font-semibold ${isFailed || isCrashed ? "sem-violation" : isAborted ? "sem-caution" : "text-secondary"}`}>
                  {task?.status ?? (isCrashed ? "BROWSER_CRASHED" : "STOPPED")}
                </span>
              </div>

              {task && (
                <div className="flex items-start gap-3">
                  <span className="label-uppercase text-muted shrink-0 pt-0.5">TASK ID</span>
                  <span className="data-mono text-[11px] text-secondary break-all">{task.id}</span>
                </div>
              )}

              <div className="flex items-start gap-3">
                <span className="label-uppercase text-muted shrink-0 pt-0.5">DETAIL</span>
                <span className="text-[12px] text-secondary leading-relaxed">{secondary}</span>
              </div>

              {browser && (
                <div className="flex items-start gap-3">
                  <span className="label-uppercase text-muted shrink-0 pt-0.5">BROWSER</span>
                  <span className="data-mono text-[11px] text-secondary">
                    {browser.state}{browser.reason ? ` · ${browser.reason}` : ""}
                  </span>
                </div>
              )}

              {state.verification && (
                <div className="flex items-start gap-3">
                  <span className="label-uppercase text-muted shrink-0 pt-0.5">LAST VERIFY</span>
                  <span className="data-mono text-[11px] text-secondary">
                    {state.verification.outcome} · {state.verification.evidenceCount} evidence items
                  </span>
                </div>
              )}
            </div>
          </section>

          {/* Safety notice */}
          <section className="border-l-2 border-line bg-surface-1 py-3 pl-4 pr-5">
            <p className="text-[12px] text-secondary leading-relaxed">
              Autonomous execution has stopped. No further browser actions will be dispatched
              by the agent for this task. The audit trail has been preserved. Review the Audit
              screen for the full record.
            </p>
            {(isAborted || isFailed) && (
              <p className="mt-2 text-[12px] text-muted leading-relaxed">
                If the outcome is uncertain, do not assume success or failure. Check the page
                state in the managed browser before making further decisions.
              </p>
            )}
          </section>

          {/* Actions */}
          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={handleDismiss}
              className="
                border-1px line-strong bg-surface-3 px-6 py-2
                text-[12.5px] font-semibold text-primary uppercase tracking-wider-safe
                hover:bg-bg-3 active:bg-surface-2
                transition-colors duration-120 max-radius-2
              "
            >
              NEW TASK
            </button>

            <button
              type="button"
              onClick={() => void handleRefreshHistory()}
              className="
                border-1px line bg-surface-1 px-5 py-2
                text-[12.5px] text-primary uppercase tracking-wider-safe
                hover:bg-surface-2 active:bg-surface-3
                transition-colors duration-120 max-radius-2
              "
            >
              VIEW HISTORY
            </button>

            {task && task.status === "RUNNING" && (
              <button
                type="button"
                onClick={() => void handleAbort()}
                disabled={dismissing}
                className="
                  border-1px sem-violation border-current px-5 py-2
                  text-[12.5px] sem-violation uppercase tracking-wider-safe
                  hover:bg-surface-2
                  disabled:cursor-not-allowed disabled:opacity-50
                  transition-colors duration-120 max-radius-2
                "
              >
                {dismissing ? "ABORTING..." : "FORCE ABORT"}
              </button>
            )}
          </div>

          {error && (
            <p className="text-[11px] data-mono sem-violation">{error}</p>
          )}

          {/* Navigation to audit */}
          <p className="micro-annotation text-muted">
            AUDIT TRAIL PRESERVED LOCALLY · OPEN AUDIT SCREEN FOR FULL RECORD AND CHAIN VERIFICATION
          </p>
        </div>
      </div>
    </main>
  );
});
