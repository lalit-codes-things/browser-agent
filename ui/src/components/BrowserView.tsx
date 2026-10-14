import type { BrowserState } from '../domain/types';

/**
 * The real managed Chromium page is rendered by the native host in the CEF
 * native-view composition. This component only reflects runtime-reported state;
 * no screenshot, placeholder page or unrelated WebView may substitute for the
 * real managed browser instance (Section 2.1).
 */

const SECURITY_COPY: Record<BrowserState['securityState']['state'], string> = {
  https: 'Secure connection',
  http: 'Not secure - HTTP',
  'mixed-content': 'Connection warning',
  unknown: 'Undetermined security state',
};

interface Props {
  state: BrowserState;
  /** True only when a real native host has attached its bridge. */
  bridgeAttached: boolean;
}

export function BrowserView({ state, bridgeAttached }: Props) {
  if (!bridgeAttached) {
    return (
      <div className="ba-browser-view ba-browser-view--offline">
        <div className="ba-browser-view__status">Native browser host not attached</div>
        <div className="ba-browser-view__viewport">
          <p className="ba-status">
            No managed Chromium instance is present in this build, so no page can be displayed and
            no navigation state is reported.
          </p>
          <p className="ba-status ba-status--muted">
            The previous native host was removed and is being rewritten from the locked
            specification. This screen reports absence instead of simulating a page.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="ba-browser-view" data-security={state.securityState.state}>
      <div className="ba-browser-view__status">{SECURITY_COPY[state.securityState.state]}</div>
      <div className="ba-browser-view__viewport">
        {state.loading ? (
          <p className="ba-status">Loading</p>
        ) : (
          <p className="ba-status">Managed page content renders here.</p>
        )}
        <p className="ba-status ba-status--muted">URL: {state.currentUrl}</p>
      </div>
    </div>
  );
}
