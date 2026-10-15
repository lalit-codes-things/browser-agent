import { SCHEMA_VERSION } from '../../bridge/protocol';
import { SHORTCUTS, UNIMPLEMENTED_SHORTCUTS, shortcutText } from '../../domain/useShortcuts';
import { useRuntime } from '../../domain/useRuntime';

/** Stated so the documentation matches what was actually exercised. */
export const MEASURED_SHORTCUTS_NOTE =
  'These are the shortcuts this build handles; the list is generated from the keyboard handler, so no unsupported hint is shown.';

/**
 * Settings (Section 2.8, Section 18.2).
 *
 * The required shortcut documentation lives here, generated from the same table
 * the keyboard handler uses, so a documented shortcut is one that exists. The
 * privacy and network facts are stated as they are configured, including the
 * ones that are not implemented yet.
 */
export function SettingsView() {
  const runtime = useRuntime();

  return (
    <div className="ba-secondary-body">
      <h3 className="ba-secondary-subheading">Keyboard shortcuts</h3>
      <table className="ba-table">
        <caption className="ba-visually-hidden">
          Implemented shortcuts with their platform key names
        </caption>
        <thead>
          <tr>
            <th scope="col">Action</th>
            <th scope="col">Keys</th>
          </tr>
        </thead>
        <tbody>
          {SHORTCUTS.map((shortcut) => (
            <tr key={shortcut.id}>
              <td className="ba-table-primary">{shortcut.description}</td>
              <td>{shortcutText(shortcut)}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <p className="ba-status ba-status--muted">{MEASURED_SHORTCUTS_NOTE}</p>

      <h3 className="ba-secondary-subheading">Not implemented in this build</h3>
      <ul className="ba-plain-list">
        {UNIMPLEMENTED_SHORTCUTS.map((entry) => (
          <li key={entry.label} className="ba-plain-list__item">
            <span className="ba-table-primary">{entry.label}</span>
            <span className="ba-table-secondary">{entry.reason}</span>
          </li>
        ))}
      </ul>

      <h3 className="ba-secondary-subheading">Privacy and network</h3>
      <dl className="ba-about">
        <dt>Inference</dt>
        <dd>
          Local only. The model runs on this computer; no cloud model endpoint or API key is used,
          and no task text leaves the machine for inference.
        </dd>
        <dt>Browser data</dt>
        <dd>
          The managed browser uses its own app-owned profile and never attaches to, reuses or
          migrates from an ordinary Chrome profile.
        </dd>
        <dt>Remote debugging</dt>
        <dd>
          No CDP endpoint and no local HTTP control listener exist in a release build. Attaching one
          is a hard-stop condition.
        </dd>
        <dt>Remote page access to the interface</dt>
        <dd>
          Website frames are never given the app's native bridge; only the first-party interface
          origin holds it.
        </dd>
        <dt>Runtime connection</dt>
        <dd>
          {runtime.runtimeConnected
            ? `Connected to ${runtime.hostLabel ?? 'the host adapter'}; interface schema version ${SCHEMA_VERSION}.`
            : 'Not connected, so nothing is inferred, navigated or verified in this window.'}
        </dd>
      </dl>
    </div>
  );
}
