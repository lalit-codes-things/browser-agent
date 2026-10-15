import type { BrowserChromeState } from '../domain/useBrowser';

/**
 * The managed Chromium page is rendered by the native host inside the CEF
 * native-view composition. This component owns the region's *reporting* only: it
 * shows what the host reported about the active tab. No screenshot, placeholder
 * page or unrelated WebView may substitute for the real managed browser instance
 * (Section 2.1, [BA-CEF-001]).
 *
 * The security label and loading state are the host's own strings, so the UI
 * never upgrades "Not secure" to a reassuring claim of its own.
 */

interface Props {
  state: BrowserChromeState;
  /** True only when a real host adapter supplied a validated channel. */
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
            The CEF native host is being built from the locked specification (ADR-010). This screen
            reports absence instead of simulating a page.
          </p>
        </div>
      </div>
    );
  }

  if (state.windowId === null) {
    return (
      <div className="ba-browser-view ba-browser-view--offline">
        <div className="ba-browser-view__status">No managed window reported</div>
        <div className="ba-browser-view__viewport">
          <p className="ba-status">
            The host adapter is attached but has not reported an application window, so there is no
            page to show.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="ba-browser-view" data-private={state.private}>
      <div className="ba-browser-view__status">
        {state.securityLabel}
        {state.private ? ' — Private Window' : ''}
        {state.loading ? ' — Loading' : ''}
      </div>
      <div className="ba-browser-view__viewport">
        <p className="ba-browser-view__title">
          {state.title === '' ? 'The host has not reported a page title.' : state.title}
        </p>
        <p className="ba-status ba-status--muted">
          Address: {state.address === '' ? 'not reported' : state.address}
        </p>
        <p className="ba-status ba-status--muted">Origin: {state.origin}</p>
        <p className="ba-status ba-status--muted">
          The managed page renders in this region, in the same native window as the browser chrome
          and the Agent panel. Its pixels are the browser's, not this document's.
        </p>
      </div>
    </div>
  );
}
