// Network Screen — real egress enforcement state.
//
// The configured mode and the actual enforcement-runtime activity are
// reported separately by the backend; the frontend never infers that
// egress is enforced from a Policy decision. DESIGN.md §14.9.

import { memo, useCallback, useState } from "react";
import type { Dispatch } from "react";
import type { AppState, AppEvent } from "../../state/app";
import { getEgressStatus } from "../../ipc/client";
import { formatIpcError } from "../../ipc/errors";

const REFRESH_BUTTON = `
  border-1px line px-3 py-1
  text-[11px] text-primary uppercase tracking-wider-safe
  hover:bg-surface-2 disabled:cursor-not-allowed disabled:text-muted
  transition-colors duration-120
`;

export const NetworkScreen = memo(function NetworkScreen({
  state,
  dispatch,
}: {
  state: AppState;
  dispatch: Dispatch<AppEvent>;
}) {
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const egress = state.egress;

  const handleRefresh = useCallback(async () => {
    setRefreshing(true);
    setError(null);
    try {
      const payload = await getEgressStatus();
      dispatch({ type: "EGRESS_STATE", payload });
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setRefreshing(false);
    }
  }, [dispatch]);

  return (
    <main className="flex min-h-0 flex-1 flex-col">
      <div className="border-b-1px line px-6 py-4">
        <div className="flex items-baseline justify-between">
          <h1 className="section-header text-primary">NETWORK</h1>
          <div className="flex items-center gap-3">
            <span className="micro-annotation text-muted">
              ENFORCEMENT STATE FROM PLATFORM BACKEND
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

        {!egress ? (
          <div className="flex h-32 items-center justify-center border border-dashed border-line-strong">
            <p className="text-[12px] data-mono text-secondary">
              EGRESS STATE NOT_EMITTED
            </p>
          </div>
        ) : (
          <section className="max-w-2xl space-y-4 border-1px line bg-surface-1 p-5">
            <div className="flex items-baseline gap-3">
              <span className="label-uppercase w-40 shrink-0 text-muted">
                EGRESS MODE
              </span>
              <span className="data-mono text-[12.5px] text-primary">
                {egress.mode}
              </span>
            </div>
            <div className="flex items-baseline gap-3">
              <span className="label-uppercase w-40 shrink-0 text-muted">
                ENFORCEMENT
              </span>
              <span
                className={`data-mono text-[12.5px] font-semibold ${
                  egress.enforcementActive ? "sem-verified" : "sem-caution"
                }`}
              >
                {egress.enforcementActive ? "ACTIVE" : "NOT ACTIVE"}
              </span>
            </div>
            <div className="flex items-baseline gap-3">
              <span className="label-uppercase w-40 shrink-0 text-muted">
                REASON
              </span>
              <span className="data-mono text-[12.5px] text-secondary">
                {egress.reason ?? "—"}
              </span>
            </div>
            <p className="border-t-1px line pt-3 text-[12px] text-secondary">
              The configured mode is the intended policy. Enforcement is only
              claimed when the platform runtime (local proxy + pf helper) is
              actually engaged.
            </p>
          </section>
        )}

        <p className="mt-4 micro-annotation text-muted">
          QUIC BLOCKED · DOH DISABLED · PROXY DEATH FAILS CLOSED (CATALOG
          INVARIANTS — NOT USER-OVERRIDABLE)
        </p>
      </div>
    </main>
  );
});
