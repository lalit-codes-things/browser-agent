import { useCallback, memo, useState, useRef } from "react";
import { AppState } from "../../state/app";
import { submitTask } from "../../ipc/client";
import { formatIpcError } from "../../ipc/errors";
import { ScreenshotPreview } from "../../components/preview/ScreenshotPreview";
import { TaskLedger } from "./TaskLedger";

export const ConsoleShell = memo(function ConsoleShell({ state }: { state: AppState }) {
  const [draft, setDraft] = useState("");
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const [localError, setLocalError] = useState<string | null>(null);

  const handleSubmit = useCallback(async () => {
    if (!draft.trim()) return;
    setLocalError(null);
    try {
      await submitTask({ task_text: draft.trim() });
      setDraft("");
    } catch (err) {
      setLocalError(formatIpcError(err));
    }
  }, [draft]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      handleSubmit();
    }
  }, [handleSubmit]);

  return (
    <main className="flex min-h-0 flex-1 flex-col">
      <section className="border-b-1px line border-b px-6 py-4">
        <h1 className="mb-3 text-primary section-header">CONSOLE</h1>
        <p className="mb-4 text-muted body-text">Describe the task.</p>

        <div className="mb-4 flex max-w-3xl flex-col gap-2">
          <textarea
            ref={inputRef}
            className="
              w-full resize-none rounded-none border-1px line bg-surface-1
              px-3 py-2 text-primary body-text data-mono leading-relaxed
              placeholder:text-muted
              focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-1 focus:ring-offset-bg-0
              transition-shadow duration-120
            "
            rows={4}
            value={draft}
            onKeyDown={handleKeyDown}
            onChange={(e) => setDraft(e.target.value)}
            placeholder="Describe the task."
            aria-label="Task input"
          />
          {localError ? (
            <p className="text-sem-violation text-[12px]">{localError}</p>
          ) : null}
        </div>

        <button
          type="button"
          onClick={handleSubmit}
          disabled={!draft.trim()}
          className="
            border-1px line-strong bg-surface-2 px-6 py-2 text-primary
            font-semibold body-text uppercase tracking-wider-safe
            disabled:cursor-not-allowed disabled:opacity-50
            hover:bg-surface-3 active:bg-bg-3
            transition-colors duration-120
            max-radius-2
          "
        >
          SUBMIT TASK
        </button>
      </section>

      <div className="flex-1 flex flex-col lg:flex-row gap-6 p-6">
        <section className="flex-1 min-w-0">
          <h2 className="mb-3 text-primary section-header">TASK CONSOLE</h2>
          <TaskLedger state={state} />
        </section>

        <section className="w-full max-w-3xl lg:max-w-none">
          <h2 className="mb-3 text-primary section-header">CONTROLLED BROWSER STATE</h2>
          <ScreenshotPreview state={state} />
        </section>
      </div>
    </main>
  );
});
