// Settings Screen — local runtime settings.
//
// Only settings the backend actually persists are shown. Security-critical
// invariants (network hardening, vault key storage, audit retention,
// HIGH_STAKES threshold) render as read-only backend-owned values: the
// frontend cannot override them. DESIGN.md §14.12.

import { memo, useCallback, useEffect, useState } from "react";
import { getSettings, updateSettings } from "../../ipc/client";
import type { Settings } from "../../ipc/schemas";
import { formatIpcError } from "../../ipc/errors";

const TOGGLE_BUTTON = `
  border-1px line bg-surface-2 px-3 py-1 min-w-[5.5rem]
  text-[11px] uppercase tracking-wider-safe
  hover:bg-surface-3 disabled:cursor-not-allowed disabled:text-muted
  transition-colors duration-120
`;

const SAVE_BUTTON = `
  border-1px line-strong bg-surface-3 px-5 py-2
  text-[12px] font-semibold text-primary uppercase tracking-wider-safe
  hover:bg-bg-3 disabled:cursor-not-allowed disabled:text-muted disabled:bg-transparent
  transition-colors duration-120 max-radius-2
`;

function EditableToggle({
  label,
  value,
  onChange,
  disabled = false,
}: {
  label: string;
  value: boolean;
  onChange: (next: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <div className="flex items-center justify-between gap-4 border-b-1px line py-2.5">
      <span className="text-[12.5px] text-primary">{label}</span>
      <div className="flex items-center gap-2">
        <span className="data-mono text-[11px] text-muted">
          {value ? "ON" : "OFF"}
        </span>
        <button
          type="button"
          disabled={disabled}
          onClick={() => onChange(!value)}
          className={TOGGLE_BUTTON}
        >
          {value ? "DISABLE" : "ENABLE"}
        </button>
      </div>
    </div>
  );
}

function ReadOnlyField({
  label,
  value,
}: {
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-center justify-between gap-4 border-b-1px line py-2.5">
      <span className="text-[12.5px] text-primary">{label}</span>
      <span className="data-mono text-[11px] text-secondary">{value}</span>
    </div>
  );
}

export const SettingsScreen = memo(function SettingsScreen() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setSettings(await getSettings());
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const patch = useCallback(
    (apply: (current: Settings) => Settings) => {
      setSaved(false);
      setSettings((current) => (current ? apply(current) : current));
    },
    [],
  );

  const handleSave = useCallback(async () => {
    if (!settings) return;
    setSaving(true);
    setError(null);
    setSaved(false);
    try {
      await updateSettings({ settings });
      setSaved(true);
      // Reload so the UI reflects exactly what the backend persisted.
      await load();
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setSaving(false);
    }
  }, [settings, load]);

  return (
    <main className="flex min-h-0 flex-1 flex-col">
      <div className="border-b-1px line px-6 py-4">
        <div className="flex items-baseline justify-between">
          <h1 className="section-header text-primary">SETTINGS</h1>
          <span className="micro-annotation text-muted">
            LOCAL RUNTIME CONFIGURATION · VALIDATED IN RUST
          </span>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-6">
        {error ? (
          <div className="mb-4 border-1px line bg-surface-1 p-3">
            <p className="text-[12px] data-mono sem-violation">{error}</p>
          </div>
        ) : null}

        {loading && !settings ? (
          <div className="flex h-32 items-center justify-center border-1px line">
            <p className="text-[12px] data-mono text-muted">
              LOADING SETTINGS...
            </p>
          </div>
        ) : !settings ? (
          <div className="flex h-32 items-center justify-center border-1px line">
            <p className="text-[12px] data-mono sem-violation">
              SETTINGS UNAVAILABLE
            </p>
          </div>
        ) : (
          <div className="max-w-2xl space-y-6">
            <section className="border-1px line bg-surface-1 p-5">
              <h2 className="mb-3 label-uppercase text-muted">RUNTIME</h2>
              <EditableToggle
                label="Announce inference latency"
                value={settings.runtime.announce_latency}
                onChange={(next) =>
                  patch((c) => ({
                    ...c,
                    runtime: { ...c.runtime, announce_latency: next },
                  }))
                }
              />
              <EditableToggle
                label="Keyboard: reduced motion"
                value={settings.keyboard.reduced_motion}
                onChange={(next) =>
                  patch((c) => ({
                    ...c,
                    keyboard: { ...c.keyboard, reduced_motion: next },
                  }))
                }
              />
            </section>

            <section className="border-1px line bg-surface-1 p-5">
              <h2 className="mb-3 label-uppercase text-muted">BROWSER</h2>
              <EditableToggle
                label="Allow headless managed browser"
                value={settings.browser.headless_allowed}
                onChange={(next) =>
                  patch((c) => ({
                    ...c,
                    browser: { ...c.browser, headless_allowed: next },
                  }))
                }
              />
            </section>

            <section className="border-1px line bg-surface-1 p-5">
              <h2 className="mb-3 label-uppercase text-muted">MODEL</h2>
              <EditableToggle
                label="Use provisional pinned model"
                value={settings.model.use_provisional_model}
                onChange={(next) =>
                  patch((c) => ({
                    ...c,
                    model: { ...c.model, use_provisional_model: next },
                  }))
                }
              />
            </section>

            <section className="border-1px line bg-surface-1 p-5">
              <h2 className="mb-3 label-uppercase text-muted">NETWORK</h2>
              <EditableToggle
                label={`Egress mode (${settings.network.egress_mode})`}
                value={settings.network.egress_mode === "ENFORCED"}
                onChange={(next) =>
                  patch((c) => ({
                    ...c,
                    network: {
                      ...c.network,
                      egress_mode: next ? "ENFORCED" : "DISABLED",
                    },
                  }))
                }
              />
              <ReadOnlyField
                label="QUIC blocked (hardening invariant)"
                value={settings.network.quic_blocked ? "ENFORCED" : "OFF"}
              />
              <ReadOnlyField
                label="DoH disabled (hardening invariant)"
                value={settings.network.doh_disabled ? "ENFORCED" : "OFF"}
              />
            </section>

            <section className="border-1px line bg-surface-1 p-5">
              <h2 className="mb-3 label-uppercase text-muted">
                VAULT / SECURITY / AUDIT
              </h2>
              <ReadOnlyField
                label="Vault master key storage"
                value={settings.vault.master_key_storage}
              />
              <ReadOnlyField
                label="HIGH_STAKES threshold"
                value={
                  settings.security.high_stakes_threshold_amount ??
                  "UNDEFINED — not user-overridable"
                }
              />
              <ReadOnlyField
                label="Audit retention policy"
                value={settings.audit.retention_policy}
              />
            </section>

            <div className="flex items-center gap-3">
              <button
                type="button"
                onClick={() => void handleSave()}
                disabled={saving || !settings}
                className={SAVE_BUTTON}
              >
                {saving ? "SAVING..." : "SAVE SETTINGS"}
              </button>
              {saved ? (
                <span className="data-mono text-[11px] sem-verified">
                  SAVED — RELOADED FROM BACKEND
                </span>
              ) : null}
            </div>

            <p className="micro-annotation text-muted">
              SECURITY INVARIANTS RENDER READ-ONLY · THE FRONTEND CANNOT
              OVERRIDE THEM
            </p>
          </div>
        )}
      </div>
    </main>
  );
});
