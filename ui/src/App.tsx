import { useCallback } from 'react';
import {
  AgentProvider,
  BrowserProvider,
  RuntimeProvider,
  SecondaryViewProvider,
  useAgent,
  useBrowser,
  useSecondaryView,
  useShortcuts,
} from './domain';
import { AgentPanel } from './components/AgentPanel';
import { BrowserView } from './components/BrowserView';
import { SecondaryViewPortal } from './components/SecondaryViewPortal';
import { TabStrip } from './components/TabStrip';
import { Toolbar } from './components/Toolbar';
import type { OverflowCommandId } from './components/Toolbar';
import { AboutView } from './components/secondary/AboutView';
import { AuditView } from './components/secondary/AuditView';
import { DownloadsView } from './components/secondary/DownloadsView';
import { ModelStatusView } from './components/secondary/ModelStatusView';
import { SkillsNotesView } from './components/secondary/SkillsNotesView';
import { SettingsView } from './components/secondary/SettingsView';
import { TaskHistoryView } from './components/secondary/TaskHistoryView';
import type { SecondaryView } from './domain/types';

/**
 * Application shell. Layout contract ([BA-LAYOUT-001]): 34 px tab strip plus
 * 44 px navigation toolbar above a workspace; side-by-side above 960 logical
 * pixels and stacked below it. The Agent panel is permanent and is never a
 * drawer, modal or toggle.
 *
 * Every screen is rendered from the validated runtime event stream. When no host
 * has attached, each one states that absence instead of showing sample data.
 */

const SECONDARY_TITLES: Record<SecondaryView, string> = {
  about: 'About Browser-Agent',
  history: 'Task history',
  downloads: 'Downloads',
  model: 'Model status',
  notes: 'Skills and notes',
  audit: 'Audit and diagnostics',
  settings: 'Settings',
};

function Shell() {
  const browser = useBrowser();
  const agent = useAgent();
  const secondary = useSecondaryView();

  const navigate = useCallback(
    (url: string) => {
      browser.navigate(url);
    },
    [browser],
  );

  const onCommand = useCallback(
    (command: OverflowCommandId) => {
      secondary.open(command);
    },
    [secondary],
  );

  // The documented baseline shortcuts, wired to the commands that exist.
  useShortcuts();

  const canRunAgent = agent.runtimeConnected;
  const active = secondary.active;

  return (
    <div className="ba-shell">
      <TabStrip
        tabs={browser.state.tabs}
        activeTabId={browser.state.activeTabId}
        onSelect={(id) => browser.activateTab(id)}
        onClose={(id) => browser.closeTab(id)}
        emptyState={
          <p className="ba-tabstrip-empty">
            {browser.bridgeAttached
              ? 'No tabs are open. The managed host has not reported one yet.'
              : 'No tabs. The managed Chromium host is not attached in this build, so no page can be opened.'}
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
          onPause={canRunAgent ? agent.pause : undefined}
          onCancel={canRunAgent ? agent.cancel : undefined}
          onTakeOver={canRunAgent ? agent.takeOver : undefined}
          onResume={canRunAgent ? agent.resume : undefined}
          onExtendBudget={canRunAgent ? agent.extendBudget : undefined}
          onFinishWithResults={canRunAgent ? agent.finishWithResults : undefined}
          onRespondToApproval={canRunAgent ? agent.respondToApproval : undefined}
        />
      </div>

      {active !== null && (
        <SecondaryViewPortal name={SECONDARY_TITLES[active]} onClose={secondary.close}>
          {active === 'about' && <AboutView />}
          {active === 'history' && <TaskHistoryView />}
          {active === 'downloads' && <DownloadsView />}
          {active === 'model' && <ModelStatusView />}
          {active === 'notes' && <SkillsNotesView />}
          {active === 'audit' && <AuditView />}
          {active === 'settings' && <SettingsView />}
        </SecondaryViewPortal>
      )}
    </div>
  );
}

export function App() {
  return (
    <RuntimeProvider>
      <BrowserProvider>
        <AgentProvider>
          <SecondaryViewProvider>
            <Shell />
          </SecondaryViewProvider>
        </AgentProvider>
      </BrowserProvider>
    </RuntimeProvider>
  );
}
