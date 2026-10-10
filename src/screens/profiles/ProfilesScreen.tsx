// Profiles Screen — managed-browser profile lifecycle.
//
// Profiles shown here are managed-browser profiles owned by the runtime,
// never the user's normal browser profile. Purge is immediate and
// backend-audited. DESIGN.md §14.11.

import { memo, useCallback, useEffect, useState } from "react";
import { listBrowserProfiles, purgeProfile } from "../../ipc/client";
import type { BrowserProfile } from "../../ipc/schemas";
import { formatIpcError } from "../../ipc/errors";

function describeKind(profile: BrowserProfile): string {
  if (profile.kind === "EPHEMERAL") return "EPHEMERAL";
  const persistent = profile.kind.PERSISTENT;
  return `PERSISTENT · ${persistent.origin}`;
}

const REFRESH_BUTTON = `
  border-1px line px-3 py-1
  text-[11px] text-primary uppercase tracking-wider-safe
  hover:bg-surface-2 disabled:cursor-not-allowed disabled:text-muted
  transition-colors duration-120
`;

const ProfileRow = memo(function ProfileRow({
  profile,
  onPurge,
}: {
  profile: BrowserProfile;
  onPurge: (id: string) => Promise<void>;
}) {
  const [purging, setPurging] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handlePurge = useCallback(async () => {
    if (!window.confirm(`Purge profile ${profile.id}? This is immediate.`)) {
      return;
    }
    setError(null);
    setPurging(true);
    try {
      await onPurge(profile.id);
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setPurging(false);
    }
  }, [profile.id, onPurge]);

  const kindLabel = describeKind(profile);
  const expires =
    profile.kind === "EPHEMERAL"
      ? "—"
      : (profile.kind.PERSISTENT.expires_at ?? "—");

  return (
    <tr className="border-b-1px line hover:bg-surface-2 transition-colors duration-120 align-top">
      <td className="px-3 py-3">
        <span className="data-mono text-[11px] text-primary break-all">
          {profile.id}
        </span>
      </td>
      <td className="px-3 py-3">
        <span
          className={`data-mono text-[11px] ${
            profile.kind === "EPHEMERAL" ? "text-secondary" : "text-primary"
          }`}
        >
          {kindLabel}
        </span>
      </td>
      <td className="px-3 py-3">
        <span className="data-mono text-[11px] text-secondary">{expires}</span>
      </td>
      <td className="px-3 py-3">
        <span className="data-mono text-[11px] text-secondary">
          {profile.storage_used_bytes.toLocaleString()} B
        </span>
      </td>
      <td className="px-3 py-3">
        <button
          type="button"
          onClick={() => void handlePurge()}
          disabled={purging}
          className="border-1px border-current sem-violation px-2 py-1 text-[10px] uppercase hover:bg-surface-2 disabled:opacity-50"
        >
          {purging ? "PURGING..." : "PURGE"}
        </button>
        {error ? (
          <p className="text-[10px] data-mono sem-violation mt-1">{error}</p>
        ) : null}
      </td>
    </tr>
  );
});

export function ProfilesScreen() {
  const [profiles, setProfiles] = useState<BrowserProfile[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setProfiles(await listBrowserProfiles());
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const handlePurge = useCallback(
    async (id: string) => {
      await purgeProfile({ id });
      await load();
    },
    [load],
  );

  return (
    <main className="flex min-h-0 flex-1 flex-col">
      <div className="border-b-1px line px-6 py-4">
        <div className="flex items-baseline justify-between">
          <h1 className="section-header text-primary">PROFILES</h1>
          <div className="flex items-center gap-3">
            <span className="micro-annotation text-muted">
              MANAGED BROWSER PROFILES · NEVER THE USER'S NORMAL PROFILE
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
              LOADING PROFILE STATE...
            </p>
          </div>
        ) : profiles.length === 0 && !error ? (
          <div className="flex h-32 items-center justify-center border-1px line">
            <div className="text-center">
              <p className="text-[12px] data-mono text-secondary">
                NO MANAGED PROFILES
              </p>
              <p className="mt-1 micro-annotation text-muted">
                Profiles are created per task by the browser runtime.
              </p>
            </div>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full border-collapse border-1px line text-left">
              <thead>
                <tr className="border-b-1px line bg-surface-2">
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">
                    ID
                  </th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">
                    KIND
                  </th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">
                    EXPIRES
                  </th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">
                    STORAGE USED
                  </th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">
                    ACTIONS
                  </th>
                </tr>
              </thead>
              <tbody>
                {profiles.map((p) => (
                  <ProfileRow key={p.id} profile={p} onPurge={handlePurge} />
                ))}
              </tbody>
            </table>
          </div>
        )}

        <p className="mt-4 micro-annotation text-muted">
          PURGE IS IMMEDIATE AND AUDITED · EPHEMERAL PROFILES EXPIRE BY DESIGN
        </p>
      </div>
    </main>
  );
}
