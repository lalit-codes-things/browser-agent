import { BrowserState } from '../domain';

/**
 * The real managed Chromium page is rendered by CEF in the native composition.
 * This component reflects the authoritative runtime state so the app reads as a
 * browser at first glance (Specification Section 2.1). No screenshot, placeholder
 * or unrelated WebView substitutes the real managed CEF browser instance.
 */
export function BrowserView({ state }: { state: BrowserState | null }) {
  if (!state) {
    return (
      <div className="ba-browser-view ba-browser-view--offline">
        <p className="ba-status">Managed Chromium not yet initialized.</p>
      </div>
    );
  }

  const securityCopy: Record<string, string> = {
    https: 'Secure connection',
    http: 'Not secure - HTTP',
    'certificate-error': 'Certificate error',
    'mixed-content': 'Connection warning',
    unknown: 'Undetermined security state',
  };

  return (
    <div className="ba-browser-view" data-security={state.securityState}>
      <div className="ba-browser-view__status">
        {securityCopy[state.securityState.state] ?? state.securityState.state}
      </div>
      <div className="ba-browser-view__viewport">
        {state.loading ? (
          <p className="ba-status">Loading...</p>
        ) : (
          <p className="ba-status">Managed page content renders here.</p>
        )}
        <p className="ba-status ba-status--muted">URL: {state.currentUrl}</p>
      </div>
    </div>
  );
}
