import { useCallback, memo, useState } from "react";
import { AppState } from "../../state/app";
import { goBack, goForward, navigateTo, reloadPage, submitTask } from "../../ipc/client";
import { formatIpcError } from "../../ipc/errors";
import { ScreenshotPreview } from "../../components/preview/ScreenshotPreview";
import { TaskLedger } from "./TaskLedger";

const BROWSER_BUTTON = `
  border-1px line bg-surface-2 px-3 py-1.5
  text-[11px] text-primary uppercase tracking-wider-safe
  hover:bg-surface-3 active:bg-bg-3
  disabled:cursor-not-allowed disabled:text-muted
  transition-colors duration-120 max-radius-2
`;

// Browser controls (DESIGN.md §6, §14.2): every control acts on the real
// managed browser through the backend. Backend unavailability renders as
// an explicit error, never as a simulated success.
const BrowserControls = memo(function BrowserControls({
  currentUrl,
}: {
  currentUrl?: string;
}) {
  const [draft, setDraft] = useState(currentUrl ?? "");
  const [controlError, setControlError] = useState<string | null>(null);

  const runControl = useCallback(async (op: () => Promise<void>) => {
    setControlError(null);
    try {
      await op();
    } catch (err) {
      setControlError(formatIpcError(err));
    }
  }, []);

  const handleNavigate = useCallback(() => {
    const url = draft.trim();
    if (!url) return;
    void runControl(() => navigateTo({ url }));
  }, [draft, runControl]);

  return (
    <div className="border-b-1px line bg-surface-1 px-4 py-2">
      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={() => void runControl(goBack)}
          className={BROWSER_BUTTON}
        >
          BACK
        </button>
        <button
          type="button"
          onClick={() => void runControl(goForward)}
          className={BROWSER_BUTTON}
        >
          FORWARD
        </button>
        <button
          type="button"
          onClick={() => void runControl(reloadPage)}
          className={BROWSER_BUTTON}
        >
          RELOAD
        </button>
        <div className="flex min-w-0 flex-1 items-center">
          <input
            type="text"
            aria-label="Address bar"
            className="
              w-full border-1px line bg-surface-0 px-3 py-1.5
              text-[12.5px] data-mono text-primary placeholder:text-muted
              focus:outline-none transition-colors duration-120
            "
            value={draft}
            placeholder={currentUrl ?? "https://"}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                handleNavigate();
              }
            }}
          />
          <button
            type="button"
            onClick={handleNavigate}
            disabled={!draft.trim()}
            className={`${BROWSER_BUTTON} ml-2 disabled:bg-transparent`}
          >
            GO
          </button>
        </div>
      </div>
      {controlError ? (
        <p className="mt-1.5 text-[11px] data-mono sem-violation">{controlError}</p>
      ) : null}
    </div>
  );
});

export const ConsoleShell = memo(function ConsoleShell({ state }: { state: AppState }) {
  const [draft, setDraft] = useState("");
  const [localError, setLocalError] = useState<string | null>(null);

  const handleSubmit = useCallback(async () => {
    const text = draft.trim();
    if (!text) return;
    setLocalError(null);
    try {
      await submitTask({ task_text: text });
      setDraft("");
    } catch (err) {
      setLocalError(formatIpcError(err));
    }
  }, [draft]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        void handleSubmit();
      }
    },
    [handleSubmit],
  );

  return (
    <main className="flex min-h-0 flex-1 flex-col">
      {/* Task Composer (§14.1): one multiline input, one verb. Not chat. */}
      <section className="border-b-1px line px-6 py-5">
        <div className="mb-4 flex items-baseline justify-between">
          <h1 className="section-header text-primary">TASK COMPOSER</h1>
          <span className="micro-annotation text-muted">LOCAL RUNTIME · MODEL ONLY</span>
        </div>

        <div className="max-w-3xl">
          <textarea
            className="
              w-full resize-none border-1px line bg-surface-1
              px-3 py-2 text-[12.5px] text-primary data-mono leading-relaxed
              placeholder:text-muted
              focus:outline-none
              transition-colors duration-120
            "
            rows={4}
            value={draft}
            onKeyDown={handleKeyDown}
            onChange={(e) => setDraft(e.target.value)}
            placeholder="Describe the task."
            aria-label="Task input"
          />

          <div className="mt-3 flex items-center justify-between gap-4">
            <button
              type="button"
              onClick={() => void handleSubmit()}
              disabled={!draft.trim()}
              className="
                border-1px line-strong bg-surface-3 px-6 py-2
                text-[12.5px] font-semibold text-primary uppercase tracking-wider-safe
                disabled:cursor-not-allowed disabled:border-line disabled:bg-transparent disabled:text-muted
                hover:bg-bg-3 active:bg-surface-2
                transition-colors duration-120 max-radius-2
              "
            >
              SUBMIT TASK
            </button>
            <span className="micro-annotation text-muted">ESC DISMISSES DIALOGS · ABORT IS ALWAYS REACHABLE</span>
          </div>

          {localError ? (
            <p className="mt-2 text-[12px] sem-violation">{localError}</p>
          ) : null}
        </div>
      </section>

      {/* Progress object: the orchestrator's actual task state. */}
      <section className="border-b-1px line bg-surface-1 px-6 py-3">
        <div className="flex flex-wrap items-center gap-x-6 gap-y-1 text-[11px]">
          <div className="flex items-center gap-2">
            <span className="label-uppercase text-muted">TASK</span>
            <span className="text-primary data-mono">
              {state.task ? state.task.status : "NOT_EMITTED"}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <span className="label-uppercase text-muted">STEP</span>
            <span className="text-primary data-mono">
              {state.task ? state.task.stepLabel : "—"}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <span className="label-uppercase text-muted">EPOCH</span>
            <span className="text-primary data-mono">
              {state.perception ? state.perception.epoch : "NOT_EMITTED"}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <span className="label-uppercase text-muted">POLICY</span>
            <span className="text-primary data-mono">
              {state.policy ? state.policy.verdict : "NOT_EMITTED"}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <span className="label-uppercase text-muted">VERIFY</span>
            <span className="text-primary data-mono">
              {state.verification ? state.verification.outcome : "NOT_EMITTED"}
            </span>
          </div>
          {state.policy?.reason ? (
            <span className="text-muted data-mono">REASON: {state.policy.reason}</span>
          ) : null}
        </div>
      </section>

      <div className="flex-1 p-6">
        <div className="flex flex-col gap-5 xl:flex-row">
          <section className="min-w-0 xl:w-[48%]">
            <h2 className="mb-2 section-header text-primary">ACTION LEDGER</h2>
            <TaskLedger state={state} />
          </section>

          <section className="min-w-0 flex-1">
            <h2 className="mb-2 section-header text-primary">CONTROLLED BROWSER STATE</h2>
            <BrowserControls currentUrl={state.navigation?.url} />
            <ScreenshotPreview state={state} />
          </section>
        </div>
      </div>
    </main>
  );
});
