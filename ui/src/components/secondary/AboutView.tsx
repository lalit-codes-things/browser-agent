import { SCHEMA_VERSION } from '../../bridge/protocol';
import { useRuntime } from '../../domain/useRuntime';
import { useBrowser } from '../../domain/useBrowser';
import { useAgent } from '../../domain/useAgent';

/**
 * About (Section 2.8).
 *
 * Diagnostic facts only: the real build version, whether a managed host has
 * actually attached, the host's own identity, and the readiness the runtime
 * reported. No fabricated telemetry and no invented version numbers.
 */
export function AboutView() {
  const runtime = useRuntime();
  const browser = useBrowser();
  const agent = useAgent();
  const readiness = agent.state.readiness;

  return (
    <dl className="ba-about">
      <dt>Application</dt>
      <dd>Browser-Agent {__BA_APP_VERSION__}</dd>

      <dt>First-party UI</dt>
      <dd>React and TypeScript bundle built from this checkout</dd>

      <dt>Interface schema version</dt>
      <dd>{SCHEMA_VERSION}</dd>

      <dt>Managed browser host</dt>
      <dd>
        {browser.bridgeAttached
          ? `Attached${browser.hostLabel === null ? '' : ` (${browser.hostLabel})`}`
          : 'Not attached - the CEF native host is still being built (ADR-010), so no page is displayed and no navigation state is reported.'}
      </dd>

      <dt>Active window and tab</dt>
      <dd>
        {browser.state.windowId === null
          ? 'No managed window is attached.'
          : `Window ${browser.state.windowId}, tab ${browser.state.activeTabId ?? 'none'}`}
      </dd>

      <dt>Local model runtime</dt>
      <dd>
        {readiness.label}
        {readiness.message === '' ? '' : ` - ${readiness.message}`}
      </dd>

      <dt>Runtime event stream</dt>
      <dd>
        {runtime.runtimeConnected
          ? `${runtime.snapshot.refusals} frame${runtime.snapshot.refusals === 1 ? '' : 's'} refused; accepted frames drive every state shown.`
          : 'Not connected, so no runtime state is displayed anywhere in this window.'}
      </dd>
    </dl>
  );
}
