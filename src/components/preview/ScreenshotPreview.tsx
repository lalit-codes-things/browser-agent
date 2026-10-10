import { memo } from "react";
import { AppState } from "../../state/app";

// Browser preview (DESIGN.md §14.2): a readout, not a second browser.
// Backend screenshots or controlled state representations only; no untrusted
// page HTML is ever rendered here. The preview cannot become the
// authorization surface.

function TrustStripField({ label, value }: { label: string; value: string }) {
  const unknown = value === "UNKNOWN" || value === "NOT_EMITTED" || value === "—";
  return (
    <div className="flex min-w-0 items-baseline gap-2">
      <span className="label-uppercase shrink-0 text-muted">{label}</span>
      {unknown ? (
        // TODO(security-review): confirm UNKNOWN treatment against token audit
        <span className="data-mono border border-dashed border-line-strong px-1 text-[12.5px] text-secondary">
          {value}
        </span>
      ) : (
        <span className="data-mono fixed-width-amount text-[12.5px] text-primary">{value}</span>
      )}
    </div>
  );
}

export const ScreenshotPreview = memo(function ScreenshotPreview({ state }: { state: AppState }) {
  const task = state.task;
  const navigation = state.navigation;
  const perception = state.perception;
  const browser = state.browser;

  const hasSignal = Boolean(browser || navigation || perception || task);

  return (
    <div className="border-1px line bg-surface-1">
      <div className="flex items-center justify-between border-b-1px line px-3 py-1.5">
        <span className="label-uppercase text-secondary">BROWSER PREVIEW</span>
        <span className="micro-annotation text-muted">READOUT ONLY</span>
      </div>

      {/* Readout plateau — controlled backend state, never page HTML. */}
      <div className="aspect-video border-b-1px line bg-surface-0">
        <div className="flex h-full flex-col items-center justify-center gap-1 px-4 text-center text-[12.5px] data-mono text-secondary">
          {hasSignal ? (
            <>
              <span>
                RUNTIME: {browser ? browser.state : "UNKNOWN"}
              </span>
              {navigation?.url ? <span>URL: {navigation.url}</span> : null}
              {perception ? (
                <span>
                  FRAME: {perception.frameId} · LOADER: {perception.loaderId}
                </span>
              ) : null}
            </>
          ) : (
            <>
              <span className="text-primary">NO ACTIVE BROWSER CONTEXT</span>
              <span className="micro-annotation text-muted">
                Frames appear when the runtime emits perception state.
              </span>
            </>
          )}
        </div>
      </div>

      {/* Trust strip (§14.2): ORIGIN · EPOCH · LOADER · FRAMES · PROCESS CLASS. */}
      <div className="flex flex-wrap items-center gap-x-5 gap-y-1.5 px-3 py-2 text-[11px]">
        <TrustStripField
          label="ORIGIN"
          value={task?.authority ?? navigation?.origin ?? "UNKNOWN"}
        />
        <TrustStripField
          label="EPOCH"
          value={perception ? String(perception.epoch) : "UNKNOWN"}
        />
        <TrustStripField
          label="LOADER"
          value={perception?.loaderId ?? "—"}
        />
        <TrustStripField
          label="FRAMES"
          value={perception?.frameId ?? "—"}
        />
        <TrustStripField
          label="ACTIONABLE"
          value={perception ? String(perception.actionableCount) : "—"}
        />
      </div>

      <div className="border-t-1px line px-3 py-1.5">
        <span className="micro-annotation text-muted">
          The preview cannot become the authorization surface.
        </span>
      </div>
    </div>
  );
});
