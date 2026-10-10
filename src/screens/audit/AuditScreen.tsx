// Audit Screen — local audit chain viewer.
//
// Queries actual audit segments from the backend store and verifies chain
// integrity. The frontend never infers integrity from task success.
// DESIGN.md §17: verified/tampered/not-verifiable explicit outcomes.

import { memo, useCallback, useEffect, useState } from "react";
import { getAuditRecords, verifyAuditChain } from "../../ipc/client";
import type { AuditSegment, ChainVerificationResult } from "../../ipc/schemas";
import { formatIpcError } from "../../ipc/errors";

function chainStatusLabel(result: ChainVerificationResult | null): {
  label: string;
  tint: string;
} {
  if (!result) return { label: "NOT_QUERIED", tint: "text-muted" };
  const s = result.status;
  if (s === "Verified") return { label: "VERIFIED INTACT", tint: "sem-verified" };
  if (s === "NotVerifiable") return { label: "NOT VERIFIABLE", tint: "text-secondary" };
  if (typeof s === "object" && "Tampered" in s) {
    return { label: `TAMPERING DETECTED AT SEQ ${s.Tampered.at}`, tint: "sem-violation" };
  }
  return { label: "UNKNOWN", tint: "text-muted" };
}

const AuditRow = memo(function AuditRow({ seg }: { seg: AuditSegment }) {
  return (
    <tr className="border-b-1px line hover:bg-surface-2 transition-colors duration-120 align-top font-mono">
      <td className="px-3 py-2 data-mono text-[11px] text-secondary">{seg.sequence}</td>
      <td className="px-3 py-2 data-mono text-[10px] text-muted" title={seg.segment_id}>
        {seg.segment_id.slice(0, 16)}...
      </td>
      <td className="px-3 py-2 data-mono text-[10px] text-muted" title={seg.hmac_link}>
        {seg.hmac_link.slice(0, 20)}...
      </td>
      <td className="px-3 py-2 data-mono text-[10px] text-muted" title={seg.segment_hmac}>
        {seg.segment_hmac.slice(0, 20)}...
      </td>
    </tr>
  );
});

export function AuditScreen() {
  const [segments, setSegments] = useState<AuditSegment[]>([]);
  const [chainResult, setChainResult] = useState<ChainVerificationResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [verifying, setVerifying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadSegments = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const segs = await getAuditRecords();
      setSegments(segs);
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void loadSegments(); }, [loadSegments]);

  const handleVerify = useCallback(async () => {
    setVerifying(true);
    setError(null);
    try {
      const result = await verifyAuditChain();
      setChainResult(result);
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setVerifying(false);
    }
  }, []);

  const chainStatus = chainStatusLabel(chainResult);

  return (
    <main className="flex min-h-0 flex-1 flex-col">
      <div className="border-b-1px line px-6 py-4">
        <div className="flex items-baseline justify-between">
          <h1 className="section-header text-primary">AUDIT CHAIN</h1>
          <span className="micro-annotation text-muted">
            APPEND-ONLY LOCAL RECORD · HMAC-LINKED SEGMENTS
          </span>
        </div>
      </div>

      {/* Chain verification bar */}
      <section className="border-b-1px line bg-surface-1 px-6 py-3">
        <div className="flex flex-wrap items-center gap-x-6 gap-y-2">
          <div className="flex items-center gap-2">
            <span className="label-uppercase text-muted text-[10px]">CHAIN STATUS</span>
            <span className={`data-mono font-semibold text-[11px] ${chainStatus.tint}`}>
              {chainStatus.label}
            </span>
          </div>
          {chainResult?.verified_at && (
            <div className="flex items-center gap-2">
              <span className="label-uppercase text-muted text-[10px]">VERIFIED AT</span>
              <span className="data-mono text-[11px] text-secondary">{chainResult.verified_at}</span>
            </div>
          )}
          <div className="flex items-center gap-2">
            <span className="label-uppercase text-muted text-[10px]">SEGMENTS</span>
            <span className="data-mono text-[11px] text-primary">{segments.length}</span>
          </div>

          <div className="ml-auto flex items-center gap-2">
            <button
              type="button"
              onClick={() => void handleVerify()}
              disabled={verifying || loading}
              className="
                border-1px line px-4 py-1.5
                text-[11px] text-primary uppercase tracking-wider-safe
                hover:bg-surface-2 disabled:cursor-not-allowed disabled:text-muted
                transition-colors duration-120
              "
            >
              {verifying ? "VERIFYING..." : "VERIFY CHAIN"}
            </button>
            <button
              type="button"
              onClick={() => void loadSegments()}
              disabled={loading}
              className="
                border-1px line px-4 py-1.5
                text-[11px] text-primary uppercase tracking-wider-safe
                hover:bg-surface-2 disabled:cursor-not-allowed disabled:text-muted
                transition-colors duration-120
              "
            >
              {loading ? "LOADING..." : "REFRESH"}
            </button>
          </div>
        </div>
      </section>

      <div className="flex-1 overflow-auto p-6">
        {error && (
          <div className="mb-4 border-1px line bg-surface-1 p-3">
            <p className="text-[12px] data-mono sem-violation">{error}</p>
          </div>
        )}

        {loading ? (
          <div className="flex h-32 items-center justify-center border-1px line">
            <p className="text-[12px] data-mono text-muted">LOADING AUDIT SEGMENTS...</p>
          </div>
        ) : segments.length === 0 ? (
          <div className="flex h-32 items-center justify-center border-1px line">
            <div className="text-center">
              <p className="text-[12px] data-mono text-secondary">NO AUDIT SEGMENTS</p>
              <p className="mt-1 micro-annotation text-muted">
                Segments are appended during task execution.
              </p>
            </div>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full border-collapse border-1px line text-left">
              <thead>
                <tr className="border-b-1px line bg-surface-2">
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">SEQ</th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">SEGMENT ID</th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">CHAIN LINK</th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">SEGMENT HMAC</th>
                </tr>
              </thead>
              <tbody>
                {segments.map((seg) => (
                  <AuditRow key={seg.segment_id} seg={seg} />
                ))}
              </tbody>
            </table>
          </div>
        )}

        <div className="mt-4 space-y-1">
          <p className="micro-annotation text-muted">
            CHAIN VERIFICATION IS BACKEND-EXECUTED · FRONTEND DISPLAYS RESULT ONLY
          </p>
          <p className="micro-annotation text-muted">
            NOT VERIFIABLE = CHAIN IS EMPTY OR HMAC KEY UNAVAILABLE · DOES NOT IMPLY TAMPERING
          </p>
        </div>
      </div>
    </main>
  );
}
