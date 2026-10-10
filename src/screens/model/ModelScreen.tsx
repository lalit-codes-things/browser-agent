// Model Screen — real model artifact state.
//
// Values come from the backend MODEL_STATE projection (pinned manifest +
// installed-artifact check). The frontend never invents verification,
// residency, or integrity claims. DESIGN.md §14.8.

import { memo, useCallback, useState } from "react";
import type { Dispatch } from "react";
import type { AppState, AppEvent } from "../../state/app";
import { getModelStatus } from "../../ipc/client";
import { formatIpcError } from "../../ipc/errors";

const REFRESH_BUTTON = `
  border-1px line px-3 py-1
  text-[11px] text-primary uppercase tracking-wider-safe
  hover:bg-surface-2 disabled:cursor-not-allowed disabled:text-muted
  transition-colors duration-120
`;

function Field({
  label,
  value,
  mono = true,
}: {
  label: string;
  value: string;
  mono?: boolean;
}) {
  const unknown = value === "UNKNOWN" || value === "NOT_EMITTED";
  return (
    <div className="flex items-baseline gap-3">
      <span className="label-uppercase w-40 shrink-0 text-muted">{label}</span>
      {unknown ? (
        <span className="data-mono border border-dashed border-line-strong px-1 text-[12.5px] text-secondary">
          {value}
        </span>
      ) : (
        <span
          className={`${mono ? "data-mono" : ""} break-all text-[12.5px] text-primary`}
        >
          {value}
        </span>
      )}
    </div>
  );
}

export const ModelScreen = memo(function ModelScreen({
  state,
  dispatch,
}: {
  state: AppState;
  dispatch: Dispatch<AppEvent>;
}) {
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const model = state.model;

  const handleRefresh = useCallback(async () => {
    setRefreshing(true);
    setError(null);
    try {
      const payload = await getModelStatus();
      dispatch({ type: "MODEL_STATE", payload });
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setRefreshing(false);
    }
  }, [dispatch]);

  const availabilityNote = model
    ? model.availability === "VERIFIED_LOADED"
      ? "Artifact loaded; SHA-256 verified at load."
      : model.availability === "PRESENT_UNVERIFIED"
        ? "Artifact bytes present. Integrity verification runs at load."
        : "Reasoning cannot run until the pinned artifact is installed."
    : "Model state has not been emitted by the backend.";

  return (
    <main className="flex min-h-0 flex-1 flex-col">
      <div className="border-b-1px line px-6 py-4">
        <div className="flex items-baseline justify-between">
          <h1 className="section-header text-primary">MODEL</h1>
          <div className="flex items-center gap-3">
            <span className="micro-annotation text-muted">
              PINNED ARTIFACT STATE · BACKEND-AUTHORITATIVE
            </span>
            <button
              type="button"
              onClick={() => void handleRefresh()}
              disabled={refreshing}
              className={REFRESH_BUTTON}
            >
              {refreshing ? "READING..." : "REFRESH"}
            </button>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-6">
        {error ? (
          <div className="mb-4 border-1px line bg-surface-1 p-3">
            <p className="text-[12px] data-mono sem-violation">{error}</p>
          </div>
        ) : null}

        <section className="max-w-2xl space-y-3 border-1px line bg-surface-1 p-5">
          <Field label="MODEL" value={model?.modelId ?? "NOT_EMITTED"} />
          <Field
            label="QUANTIZATION"
            value={model?.quantization ?? "NOT_EMITTED"}
          />
          <Field label="SHA-256" value={model?.sha256 ?? "NOT_EMITTED"} />
          <Field label="RESIDENCY" value={model?.residency ?? "NOT_EMITTED"} />
          <Field
            label="AVAILABILITY"
            value={model?.availability ?? "NOT_EMITTED"}
          />
          <Field label="REASON" value={model?.reason ?? "—"} mono={false} />
          <div className="border-t-1px line pt-3">
            <p className="text-[12px] text-secondary">{availabilityNote}</p>
          </div>
        </section>

        <p className="mt-4 micro-annotation text-muted">
          NO CLOUD INFERENCE · NO API KEYS · MODEL FILES ARE NEVER DOWNLOADED
          AT RUNTIME
        </p>
      </div>
    </main>
  );
});
