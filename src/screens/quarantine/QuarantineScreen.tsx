// Quarantine Screen — quarantined downloads and verification.
//
// Downloads are never automatically opened. Verification is a backend
// operation; the frontend only reflects backend verification state.
// DESIGN.md §14.10.

import { memo, useCallback, useEffect, useState } from "react";
import {
  listQuarantinedDownloads,
  verifyQuarantinedDownload,
} from "../../ipc/client";
import type { QuarantinedDownload } from "../../ipc/schemas";
import { formatIpcError } from "../../ipc/errors";

type DownloadVerificationState = QuarantinedDownload["verification_state"];

function stateTint(state: DownloadVerificationState): string {
  switch (state) {
    case "VERIFIED":
      return "sem-verified";
    case "REJECTED":
      return "sem-violation";
    default:
      return "sem-caution";
  }
}

const REFRESH_BUTTON = `
  border-1px line px-3 py-1
  text-[11px] text-primary uppercase tracking-wider-safe
  hover:bg-surface-2 disabled:cursor-not-allowed disabled:text-muted
  transition-colors duration-120
`;

const VERIFY_BUTTON = `
  border-1px line-strong bg-surface-3 px-3 py-1
  text-[10px] font-semibold text-primary uppercase tracking-wider-safe
  hover:bg-bg-3 disabled:cursor-not-allowed disabled:text-muted
  transition-colors duration-120
`;

const DownloadRow = memo(function DownloadRow({
  download,
  onVerify,
}: {
  download: QuarantinedDownload;
  onVerify: (id: string) => Promise<void>;
}) {
  const [verifying, setVerifying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleVerify = useCallback(async () => {
    setError(null);
    setVerifying(true);
    try {
      await onVerify(download.id);
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setVerifying(false);
    }
  }, [download.id, onVerify]);

  return (
    <tr className="border-b-1px line hover:bg-surface-2 transition-colors duration-120 align-top">
      <td className="px-3 py-3">
        <span className="data-mono text-[11px] text-primary break-all">
          {download.file_path}
        </span>
      </td>
      <td className="px-3 py-3">
        <span className="data-mono text-[11px] text-secondary">
          {download.size_bytes.toLocaleString()} B
        </span>
      </td>
      <td className="px-3 py-3">
        <span className="data-mono text-[10px] text-secondary">
          {download.declared_type ?? "UNKNOWN"}
        </span>
        <p className="text-[10px] text-muted mt-0.5">DECLARED BY SERVER</p>
      </td>
      <td className="px-3 py-3">
        <span
          className={`data-mono text-[11px] font-semibold ${stateTint(download.verification_state)}`}
        >
          {download.verification_state}
        </span>
      </td>
      <td className="px-3 py-3">
        {download.verification_state === "QUARANTINED" ? (
          <button
            type="button"
            onClick={() => void handleVerify()}
            disabled={verifying}
            className={VERIFY_BUTTON}
          >
            {verifying ? "VERIFYING..." : "VERIFY"}
          </button>
        ) : null}
        {error ? (
          <p className="text-[10px] data-mono sem-violation mt-1">{error}</p>
        ) : null}
      </td>
    </tr>
  );
});

export function QuarantineScreen() {
  const [downloads, setDownloads] = useState<QuarantinedDownload[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setDownloads(await listQuarantinedDownloads());
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const handleVerify = useCallback(
    async (id: string) => {
      await verifyQuarantinedDownload({ id });
      await load();
    },
    [load],
  );

  return (
    <main className="flex min-h-0 flex-1 flex-col">
      <div className="border-b-1px line px-6 py-4">
        <div className="flex items-baseline justify-between">
          <h1 className="section-header text-primary">QUARANTINE</h1>
          <div className="flex items-center gap-3">
            <span className="micro-annotation text-muted">
              DOWNLOADS ARE NEVER AUTOMATICALLY OPENED
            </span>
            <button
              type="button"
              onClick={() => void load()}
              disabled={loading}
              className={REFRESH_BUTTON}
            >
              {loading ? "LOADING..." : "REFRESH"}
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

        {loading && !error ? (
          <div className="flex h-32 items-center justify-center border-1px line">
            <p className="text-[12px] data-mono text-muted">
              LOADING QUARANTINE STATE...
            </p>
          </div>
        ) : downloads.length === 0 && !error ? (
          <div className="flex h-32 items-center justify-center border-1px line">
            <div className="text-center">
              <p className="text-[12px] data-mono text-secondary">
                NO QUARANTINED DOWNLOADS
              </p>
              <p className="mt-1 micro-annotation text-muted">
                Downloads captured by the managed browser appear here.
              </p>
            </div>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full border-collapse border-1px line text-left">
              <thead>
                <tr className="border-b-1px line bg-surface-2">
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">
                    FILE
                  </th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">
                    SIZE
                  </th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">
                    TYPE
                  </th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">
                    VERIFICATION
                  </th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">
                    ACTIONS
                  </th>
                </tr>
              </thead>
              <tbody>
                {downloads.map((d) => (
                  <DownloadRow key={d.id} download={d} onVerify={handleVerify} />
                ))}
              </tbody>
            </table>
          </div>
        )}

        <p className="mt-4 micro-annotation text-muted">
          SERVER-PROVIDED CONTENT-TYPE IS NEVER TREATED AS PROOF THAT A FILE
          IS SAFE
        </p>
      </div>
    </main>
  );
}
