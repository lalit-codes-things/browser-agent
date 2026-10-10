// Vault Screen — generic Credential Vault UI.
//
// Interacts with the real backend vault subsystem (Argon2id, AEAD-encrypted
// records). The frontend receives only safe metadata (VaultItemRef) and
// never decrypted secrets through ordinary state management.
// DESIGN.md §15: vault create/unlock/lock/list/add/delete.

import { memo, useCallback, useEffect, useState } from "react";
import {
  getVaultStatus,
  createVault,
  unlockVault,
  lockVault,
  listVaultItems,
  addVaultItem,
  deleteVaultItem,
} from "../../ipc/client";
import type { VaultStatus, VaultItemRef } from "../../ipc/schemas";
import { formatIpcError } from "../../ipc/errors";

// ---- Sub-components ---------------------------------------------------------

const FORM_INPUT = `
  w-full border-1px line bg-surface-0 px-3 py-2
  text-[12.5px] data-mono text-primary placeholder:text-muted
  focus:outline-none transition-colors duration-120
`;

const ACTION_BUTTON = `
  border-1px line-strong bg-surface-3 px-5 py-2
  text-[12px] font-semibold text-primary uppercase tracking-wider-safe
  hover:bg-bg-3 disabled:cursor-not-allowed disabled:text-muted disabled:bg-transparent
  transition-colors duration-120 max-radius-2
`;

const SECONDARY_BUTTON = `
  border-1px line bg-surface-1 px-4 py-2
  text-[12px] text-primary uppercase tracking-wider-safe
  hover:bg-surface-2 disabled:cursor-not-allowed disabled:text-muted
  transition-colors duration-120 max-radius-2
`;

const DANGER_BUTTON = `
  border-1px border-current sem-violation px-4 py-2
  text-[12px] uppercase tracking-wider-safe
  hover:bg-surface-2 disabled:cursor-not-allowed disabled:opacity-50
  transition-colors duration-120 max-radius-2
`;

// Create vault panel
function CreateVaultPanel({ onCreated }: { onCreated: () => void }) {
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleCreate = useCallback(async () => {
    if (!password || password !== confirm) {
      setError("Passwords do not match or are empty.");
      return;
    }
    setError(null);
    setSubmitting(true);
    try {
      await createVault({ password });
      onCreated();
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setSubmitting(false);
    }
  }, [password, confirm, onCreated]);

  return (
    <div className="max-w-md space-y-4">
      <p className="text-[12px] text-secondary leading-relaxed">
        Create a new credential vault. The master password is hashed with Argon2id and
        never stored or transmitted. All credential data is encrypted locally.
      </p>
      <div className="space-y-3">
        <div>
          <label className="label-uppercase text-muted text-[10px] block mb-1">MASTER PASSWORD</label>
          <input
            type="password"
            className={FORM_INPUT}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="Enter master password"
            autoComplete="new-password"
          />
        </div>
        <div>
          <label className="label-uppercase text-muted text-[10px] block mb-1">CONFIRM PASSWORD</label>
          <input
            type="password"
            className={FORM_INPUT}
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
            placeholder="Repeat master password"
            autoComplete="new-password"
            onKeyDown={(e) => { if (e.key === "Enter") void handleCreate(); }}
          />
        </div>
        {error && <p className="text-[11px] data-mono sem-violation">{error}</p>}
        <button type="button" onClick={() => void handleCreate()} disabled={submitting} className={ACTION_BUTTON}>
          {submitting ? "CREATING..." : "CREATE VAULT"}
        </button>
      </div>
      <p className="micro-annotation text-muted">
        LOCAL STORAGE ONLY · Argon2id KDF · AEAD ENCRYPTION
      </p>
    </div>
  );
}

// Unlock panel
function UnlockPanel({ onUnlocked }: { onUnlocked: () => void }) {
  const [password, setPassword] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleUnlock = useCallback(async () => {
    if (!password) return;
    setError(null);
    setSubmitting(true);
    try {
      await unlockVault({ password });
      setPassword("");
      onUnlocked();
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setSubmitting(false);
    }
  }, [password, onUnlocked]);

  return (
    <div className="max-w-sm space-y-3">
      <label className="label-uppercase text-muted text-[10px] block mb-1">MASTER PASSWORD</label>
      <input
        type="password"
        className={FORM_INPUT}
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        placeholder="Enter master password"
        autoComplete="current-password"
        onKeyDown={(e) => { if (e.key === "Enter") void handleUnlock(); }}
      />
      {error && <p className="text-[11px] data-mono sem-violation">{error}</p>}
      <button type="button" onClick={() => void handleUnlock()} disabled={submitting || !password} className={ACTION_BUTTON}>
        {submitting ? "UNLOCKING..." : "UNLOCK VAULT"}
      </button>
      <p className="micro-annotation text-muted">BACKEND VERIFIES AGAINST STORED KDF HASH</p>
    </div>
  );
}

// Add item form
function AddItemForm({ onAdded }: { onAdded: () => void }) {
  const [origin, setOrigin] = useState("");
  const [label, setLabel] = useState("");
  const [secret, setSecret] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleAdd = useCallback(async () => {
    if (!origin || !label || !secret) {
      setError("All fields are required.");
      return;
    }
    setError(null);
    setSubmitting(true);
    try {
      await addVaultItem({ origin, account_label: label, secret });
      setOrigin(""); setLabel(""); setSecret("");
      onAdded();
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setSubmitting(false);
    }
  }, [origin, label, secret, onAdded]);

  return (
    <div className="border-1px line bg-surface-1 p-4 space-y-3">
      <h3 className="label-uppercase text-muted text-[10px]">ADD CREDENTIAL</h3>
      <div className="grid grid-cols-2 gap-3">
        <div>
          <label className="label-uppercase text-muted text-[10px] block mb-1">ORIGIN (URL)</label>
          <input type="text" className={FORM_INPUT} value={origin}
            onChange={(e) => setOrigin(e.target.value)} placeholder="https://example.com" />
        </div>
        <div>
          <label className="label-uppercase text-muted text-[10px] block mb-1">ACCOUNT LABEL</label>
          <input type="text" className={FORM_INPUT} value={label}
            onChange={(e) => setLabel(e.target.value)} placeholder="user@example.com" />
        </div>
        <div className="col-span-2">
          <label className="label-uppercase text-muted text-[10px] block mb-1">SECRET (PASSWORD / TOKEN)</label>
          <input type="password" className={FORM_INPUT} value={secret}
            onChange={(e) => setSecret(e.target.value)} placeholder="Credential secret"
            autoComplete="new-password" />
        </div>
      </div>
      {error && <p className="text-[11px] data-mono sem-violation">{error}</p>}
      <div className="flex items-center gap-3">
        <button type="button" onClick={() => void handleAdd()} disabled={submitting} className={ACTION_BUTTON}>
          {submitting ? "ADDING..." : "ADD CREDENTIAL"}
        </button>
      </div>
      <p className="micro-annotation text-muted">
        SECRET IS ENCRYPTED IN RUST BEFORE ANY STORAGE · NEVER PASSES THROUGH REACT STATE
      </p>
    </div>
  );
}

// Credential item row
const ItemRow = memo(function ItemRow({
  item,
  onDelete,
}: {
  item: VaultItemRef;
  onDelete: (id: string) => Promise<void>;
}) {
  const [deleting, setDeleting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleDelete = useCallback(async () => {
    if (!confirm(`Delete credential for ${item.origin}?`)) return;
    setError(null);
    setDeleting(true);
    try {
      await onDelete(item.id);
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setDeleting(false);
    }
  }, [item.id, item.origin, onDelete]);

  const httpsColor = item.https_check_state === "PASS" ? "sem-verified" : item.https_check_state === "FAIL" ? "sem-violation" : "text-muted";
  const idnColor = item.idn_homograph_check_state === "PASS" ? "sem-verified" : item.idn_homograph_check_state === "MISMATCH" ? "sem-violation" : "text-muted";

  return (
    <tr className="border-b-1px line hover:bg-surface-2 transition-colors duration-120 align-top">
      <td className="px-3 py-3">
        <span className="data-mono text-[11px] text-primary">{item.origin}</span>
      </td>
      <td className="px-3 py-3">
        <span className="text-[11px] text-secondary">{item.account_label}</span>
      </td>
      <td className="px-3 py-3">
        <span className={`data-mono text-[10px] ${httpsColor}`}>{item.https_check_state}</span>
      </td>
      <td className="px-3 py-3">
        <span className={`data-mono text-[10px] ${idnColor}`}>{item.idn_homograph_check_state}</span>
      </td>
      <td className="px-3 py-3">
        <span className="data-mono text-[10px] text-muted">
          {item.last_used_task ? item.last_used_task.slice(0, 10) + "..." : "NEVER"}
        </span>
      </td>
      <td className="px-3 py-3">
        <button
          type="button"
          onClick={() => void handleDelete()}
          disabled={deleting}
          className="border-1px border-current sem-violation px-2 py-1 text-[10px] uppercase hover:bg-surface-2 disabled:opacity-50"
        >
          {deleting ? "DELETING..." : "DELETE"}
        </button>
        {error && <p className="text-[10px] data-mono sem-violation mt-1">{error}</p>}
      </td>
    </tr>
  );
});

// ---- Main Component ----------------------------------------------------------

export function VaultScreen() {
  const [status, setStatus] = useState<VaultStatus | null>(null);
  const [items, setItems] = useState<VaultItemRef[]>([]);
  const [loading, setLoading] = useState(true);
  const [locking, setLocking] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);

  const loadStatus = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const s = await getVaultStatus();
      setStatus(s);
      if (s.is_unlocked) {
        const its = await listVaultItems();
        setItems(its);
      }
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void loadStatus(); }, [loadStatus]);

  const handleLock = useCallback(async () => {
    setLocking(true);
    setError(null);
    try {
      await lockVault();
      setItems([]);
      await loadStatus();
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setLocking(false);
    }
  }, [loadStatus]);

  const handleDelete = useCallback(async (id: string) => {
    await deleteVaultItem({ id });
    const its = await listVaultItems();
    setItems(its);
  }, []);

  return (
    <main className="flex min-h-0 flex-1 flex-col">
      <div className="border-b-1px line px-6 py-4">
        <div className="flex items-baseline justify-between">
          <h1 className="section-header text-primary">CREDENTIAL VAULT</h1>
          <div className="flex items-center gap-3">
            {status && (
              <span className={`data-mono text-[11px] ${status.is_unlocked ? "sem-verified" : "text-secondary"}`}>
                {status.is_unlocked ? "UNLOCKED" : status.is_initialized ? "LOCKED" : "NOT INITIALIZED"}
              </span>
            )}
            {status?.is_unlocked && (
              <button type="button" onClick={() => void handleLock()} disabled={locking} className={SECONDARY_BUTTON}>
                {locking ? "LOCKING..." : "LOCK VAULT"}
              </button>
            )}
            <button type="button" onClick={() => void loadStatus()} disabled={loading} className={SECONDARY_BUTTON}>
              {loading ? "..." : "REFRESH"}
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

        {loading ? (
          <p className="text-[12px] data-mono text-muted">LOADING VAULT STATUS...</p>
        ) : !status ? (
          <p className="text-[12px] data-mono sem-violation">VAULT STATUS UNAVAILABLE</p>
        ) : !status.is_initialized ? (
          <section>
            <h2 className="mb-4 section-header text-primary">INITIALIZE VAULT</h2>
            <CreateVaultPanel onCreated={() => void loadStatus()} />
          </section>
        ) : !status.is_unlocked ? (
          <section>
            <h2 className="mb-4 section-header text-primary">UNLOCK VAULT</h2>
            <UnlockPanel onUnlocked={() => void loadStatus()} />
          </section>
        ) : (
          <section className="space-y-5">
            {/* Status bar */}
            <div className="flex items-center gap-6 border-b-1px line pb-3">
              <div className="flex items-center gap-2">
                <span className="label-uppercase text-muted text-[10px]">ITEMS</span>
                <span className="data-mono text-[12px] text-primary">{status.item_count}</span>
              </div>
              <button
                type="button"
                onClick={() => setShowAddForm((v) => !v)}
                className={SECONDARY_BUTTON}
              >
                {showAddForm ? "CANCEL" : "+ ADD CREDENTIAL"}
              </button>
            </div>

            {showAddForm && (
              <AddItemForm onAdded={async () => {
                setShowAddForm(false);
                const its = await listVaultItems();
                setItems(its);
              }} />
            )}

            {items.length === 0 ? (
              <div className="flex h-32 items-center justify-center border-1px line">
                <p className="text-[12px] data-mono text-muted">NO CREDENTIALS STORED</p>
              </div>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full border-collapse border-1px line text-left">
                  <thead>
                    <tr className="border-b-1px line bg-surface-2">
                      <th className="px-3 py-2 label-uppercase text-muted text-[10px]">ORIGIN</th>
                      <th className="px-3 py-2 label-uppercase text-muted text-[10px]">ACCOUNT</th>
                      <th className="px-3 py-2 label-uppercase text-muted text-[10px]">HTTPS</th>
                      <th className="px-3 py-2 label-uppercase text-muted text-[10px]">IDN</th>
                      <th className="px-3 py-2 label-uppercase text-muted text-[10px]">LAST USED</th>
                      <th className="px-3 py-2 label-uppercase text-muted text-[10px]">ACTIONS</th>
                    </tr>
                  </thead>
                  <tbody>
                    {items.map((item) => (
                      <ItemRow key={item.id} item={item} onDelete={handleDelete} />
                    ))}
                  </tbody>
                </table>
              </div>
            )}

            <p className="micro-annotation text-muted">
              SECRETS ARE NEVER RETURNED TO REACT STATE · SECURE FILL IS AN AUTHORIZED NATIVE OPERATION
            </p>
          </section>
        )}
      </div>
    </main>
  );
}

// Quiet re-export so it can be used as named import by App.tsx
export default VaultScreen;
