// Skills Screen — real backend skill records.
//
// Lists skills from the backend store. Approval transitions are
// backend-owned. The frontend only reflects backend SkillStatus.
// DESIGN.md §16: skill status, version, origin scope, commitment hash.

import { memo, useCallback, useEffect, useState } from "react";
import {
  listSkills,
  approveSkillVersion,
} from "../../ipc/client";
import type { SkillRecord } from "../../ipc/schemas";
import { formatIpcError } from "../../ipc/errors";

type SkillStatus = SkillRecord["status"];

function statusTint(status: SkillStatus): string {
  switch (status) {
    case "ACTIVE": return "sem-verified";
    case "SHADOW": return "text-secondary";
    case "SUSPENDED_DRIFT": return "sem-caution";
    case "REVOKED": return "sem-violation";
    default: return "text-muted";
  }
}

function statusNote(status: SkillStatus): string {
  switch (status) {
    case "ACTIVE": return "Live — passes through Policy per-step";
    case "SHADOW": return "Candidate — evaluation only, no side effects";
    case "SUSPENDED_DRIFT": return "Drift detected — suspended pending review";
    case "REVOKED": return "Revoked — permanently inactive";
    default: return "";
  }
}

const SkillRow = memo(function SkillRow({
  skill,
  onApprove,
}: {
  skill: SkillRecord;
  onApprove: (id: string, version: string) => Promise<void>;
}) {
  const [approving, setApproving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleApprove = useCallback(async () => {
    setError(null);
    setApproving(true);
    try {
      await onApprove(skill.id, skill.version);
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setApproving(false);
    }
  }, [skill.id, skill.version, onApprove]);

  return (
    <tr className="border-b-1px line hover:bg-surface-2 transition-colors duration-120 align-top">
      <td className="px-3 py-3">
        <span className="data-mono text-[11px] text-secondary">{skill.id.slice(0, 14)}...</span>
      </td>
      <td className="px-3 py-3">
        <span className="data-mono text-[11px] text-primary">{skill.version}</span>
      </td>
      <td className="px-3 py-3">
        <span className="text-[11px] text-secondary max-w-[12rem] block truncate" title={skill.origin_scope}>
          {skill.origin_scope}
        </span>
      </td>
      <td className="px-3 py-3">
        <div>
          <span className={`data-mono text-[11px] font-semibold ${statusTint(skill.status)}`}>
            {skill.status}
          </span>
          <p className="text-[10px] text-muted mt-0.5">{statusNote(skill.status)}</p>
        </div>
      </td>
      <td className="px-3 py-3">
        <span className="data-mono text-[10px] text-muted" title={skill.commitment_hash}>
          {skill.commitment_hash.slice(0, 12)}...
        </span>
      </td>
      <td className="px-3 py-3">
        <div className="space-y-1">
          {skill.status === "SHADOW" && (
            <button
              type="button"
              onClick={() => void handleApprove()}
              disabled={approving}
              className="
                border-1px line bg-surface-2 px-3 py-1
                text-[10px] text-primary uppercase tracking-wider-safe
                hover:bg-surface-3 disabled:cursor-not-allowed disabled:text-muted
                transition-colors duration-120
              "
            >
              {approving ? "APPROVING..." : "APPROVE VERSION"}
            </button>
          )}
          {error && (
            <p className="text-[10px] data-mono sem-violation">{error}</p>
          )}
        </div>
      </td>
    </tr>
  );
});

export function SkillsScreen() {
  const [skills, setSkills] = useState<SkillRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await listSkills();
      setSkills(result);
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void load(); }, [load]);

  const handleApprove = useCallback(async (id: string, version: string) => {
    await approveSkillVersion({ id, version });
    await load();
  }, [load]);

  return (
    <main className="flex min-h-0 flex-1 flex-col">
      <div className="border-b-1px line px-6 py-4">
        <div className="flex items-baseline justify-between">
          <h1 className="section-header text-primary">SKILLS</h1>
          <div className="flex items-center gap-3">
            <span className="micro-annotation text-muted">
              SHADOW → ACTIVE REQUIRES EXPLICIT APPROVAL
            </span>
            <button
              type="button"
              onClick={() => void load()}
              disabled={loading}
              className="
                border-1px line px-3 py-1
                text-[11px] text-primary uppercase tracking-wider-safe
                hover:bg-surface-2 disabled:cursor-not-allowed disabled:text-muted
                transition-colors duration-120
              "
            >
              {loading ? "LOADING..." : "REFRESH"}
            </button>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-6">
        {error && (
          <div className="mb-4 border-1px line bg-surface-1 p-3">
            <p className="text-[12px] data-mono sem-violation">{error}</p>
          </div>
        )}

        {loading && !error ? (
          <div className="flex h-32 items-center justify-center border-1px line">
            <p className="text-[12px] data-mono text-muted">LOADING SKILLS FROM BACKEND...</p>
          </div>
        ) : skills.length === 0 && !error ? (
          <div className="flex h-32 items-center justify-center border-1px line">
            <div className="text-center">
              <p className="text-[12px] data-mono text-secondary">NO SKILLS RECORDED</p>
              <p className="mt-1 micro-annotation text-muted">
                Skills are recorded automatically during task execution.
              </p>
            </div>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full border-collapse border-1px line text-left">
              <thead>
                <tr className="border-b-1px line bg-surface-2">
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">ID</th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">VERSION</th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">ORIGIN SCOPE</th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">STATUS</th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">COMMITMENT</th>
                  <th className="px-3 py-2 label-uppercase text-muted text-[10px]">ACTIONS</th>
                </tr>
              </thead>
              <tbody>
                {skills.map((skill) => (
                  <SkillRow key={`${skill.id}-${skill.version}`} skill={skill} onApprove={handleApprove} />
                ))}
              </tbody>
            </table>
          </div>
        )}

        <div className="mt-4 space-y-1">
          <p className="micro-annotation text-muted">
            SKILL STATUS IS BACKEND-AUTHORITATIVE · SHADOW SKILLS RUN IN EVALUATION MODE ONLY
          </p>
          <p className="micro-annotation text-muted">
            EVERY SKILL STEP PASSES THROUGH POLICY AND CURRENT BROWSER-STATE VALIDATION
          </p>
        </div>
      </div>
    </main>
  );
}
