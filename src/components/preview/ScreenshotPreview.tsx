import { memo } from "react";
import { AppState } from "../../state/app";

// Browser preview is a readout, not a second browser.
// We only render backend-provided screenshots or controlled state
// representations. Never inject untrusted page HTML into the frontend.
// (DESIGN.md #14.2)

export const ScreenshotPreview = memo(function ScreenshotPreview({ state }: { state: AppState }) {
  const task = state.task;

  return (
    <div className="rounded-none border-1px line bg-surface-1 p-4">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-muted text-[11px] uppercase tracking-wider-safe">BROWSER PREVIEW</h3>
        <span className="text-muted text-[11px] data-mono">READOUT ONLY</span>
      </div>

      <div className="aspect-video border-1px line-strong bg-surface-2 overflow-hidden">
        <div className="flex h-full items-center justify-center text-muted text-[12px] data-mono">
          {task ? (
            <>
              <span className="block leading-relaxed">
                ORIGIN: {task.authority ?? "UNKNOWN"}
              </span>
              <span className="block leading-relaxed">
                EPOCH: {task.progress ?? "UNKNOWN"}
              </span>
            </>
          ) : (
            "NO TASK ACTIVE"
          )}
        </div>
      </div>

      <div className="mt-3 flex flex-wrap gap-x-6 gap-y-1 text-[12px]">
        <div className="flex items-center gap-2">
          <span className="text-muted label-uppercase">ORIGIN</span>
          <span className="text-primary data-mono">{task?.authority ?? "UNKNOWN"}</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-muted label-uppercase">EPOCH</span>
          <span className="text-primary data-mono">{task?.progress ?? "UNKNOWN"}</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-muted label-uppercase">LOADER</span>
          <span className="text-muted data-mono">—</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-muted label-uppercase">FRAMES</span>
          <span className="text-muted data-mono">—</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-muted label-uppercase">PROCESS CLASS</span>
          <span className="text-muted data-mono">—</span>
        </div>
      </div>

      <p className="mt-4 border-t-1px line pt-3 text-muted text-[11px] leading-relaxed">
        The preview cannot become the authorization surface.
      </p>
    </div>
  );
});
