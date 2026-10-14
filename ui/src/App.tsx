import { useCallback } from 'react';
import {
  AgentProvider,
  BrowserProvider,
  READINESS_COPY,
  SecondaryViewProvider,
  createBrowserState,
  useAgent,
  useBrowser,
  useSecondaryView,
} from './domain';
import type { AgentReadinessStatus } from './domain/types';
import { AgentPanel } from './components/AgentPanel';
import { BrowserView } from './components/BrowserView';
import { SecondaryViewPortal } from './components/SecondaryViewPortal';
import { TabStrip } from './components/TabStrip';
import { Toolbar } from './components/Toolbar';
import type { OverflowCommandId } from './components/Toolbar';

/**
 * Application shell. Layout contract ([BA-LAYOUT-001]): 34 px tab strip plus
 * 44 px navigation toolbar above a workspace; side-by-side above 960 logical
 * pixels and stacked below it. The Agent panel is permanent and is never a
 * drawer, modal or toggle.
 */
function Shell() {
  const browser = useBrowser();
  const agent = useAgent();
  const secondary = useSecondaryView();

  const navigate = useCallback(
    (url: string) => {
      const activeTabId = browser.state.activeTabId;
      if (activeTabId === null) {
        // The runtime owns tab creation. Forward the navigation and do not
        // invent a tab record the runtime did not report.
        void browser.navigate(url);
        return;
      }
      browser.navigateTab(activeTabId, url);
    },
    [browser],
  );

  const onCommand = useCallback(
    (command: OverflowCommandId) => {
      secondary.open(command);
    },
    [secondary],
  );

  const canRunAgent = agent.runtimeConnected;

  return (
    <div className="ba-shell">
      <TabStrip
        tabs={browser.state.tabs}
        activeTabId={browser.state.activeTabId}
        onSelect={(id) => browser.selectTab(id)}
        onClose={(id) => browser.closeTab(id)}
        emptyState={
          <p className="ba-tabstrip-empty">
            No tabs. The managed Chromium host is not attached in this build, so no page can be
            opened.
          </p>
        }
      />

      <Toolbar browser={browser} onNavigate={navigate} onCommand={onCommand} />

      <div className="ba-workspace">
        <BrowserView state={browser.state} bridgeAttached={browser.bridgeAttached} />

        <AgentPanel
          agent={agent.state}
          runtimeConnected={agent.runtimeConnected}
          notice={agent.notice}
          onSubmit={agent.submitTask}
          onPause={canRunAgent ? () => void agent.pause() : undefined}
          onCancel={canRunAgent ? () => void agent.cancel() : undefined}
          onTakeOver={canRunAgent ? () => void agent.takeOver() : undefined}
          onResume={canRunAgent ? () => void agent.resume() : undefined}
          onRespondToApproval={
            canRunAgent
              ? (actionId, approved) => void agent.respondToApproval(actionId, approved)
              : undefined
          }
        />
      </div>

      {secondary.active !== null && (
        <SecondaryViewPortal name="About Browser-Agent" onClose={secondary.close}>
          <AboutContent
            readiness={agent.state.readiness.status}
            readinessDetail={agent.state.readiness.message}
            bridgeAttached={browser.bridgeAttached}
          />
        </SecondaryViewPortal>
      )}
    </div>
  );
}

/**
 * Diagnostic facts only: real artifact identifiers and reported runtime
 * attachment. No fabricated telemetry and no invented version numbers
 * (Section 0.2).
 */
function AboutContent({
  readiness,
  readinessDetail,
  bridgeAttached,
}: {
  readiness: AgentReadinessStatus;
  readinessDetail: string;
  bridgeAttached: boolean;
}) {
  return (
    <dl className="ba-about">
      <dt>Application</dt>
      <dd>Browser-Agent {__BA_APP_VERSION__}</dd>

      <dt>First-party UI</dt>
      <dd>React and TypeScript bundle built from this checkout</dd>

      <dt>Native browser host (CEF)</dt>
      <dd>
        {bridgeAttached
          ? 'Attached'
          : 'Not attached - removed with the previous native stack and being rewritten (ADR-010)'}
      </dd>

      <dt>Local model runtime (llama.cpp)</dt>
      <dd>
        {readinessDetail === ''
          ? READINESS_COPY[readiness]
          : `${READINESS_COPY[readiness]} - ${readinessDetail}`}
      </dd>
    </dl>
  );
}

export function App() {
  return (
    <SecondaryViewProvider>
      <BrowserProvider initial={createBrowserState()} onBridgeReady={() => undefined}>
        <AgentProvider>
          <Shell />
        </AgentProvider>
      </BrowserProvider>
    </SecondaryViewProvider>
  );
}
