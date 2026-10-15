import { createContext, useContext, useMemo } from 'react';
import type { ReactNode } from 'react';
import { commands } from '../bridge/client';
import type { CommandBody } from '../bridge/protocol';
import { useRuntime } from './useRuntime';
import type { TabSummary } from './types';

/**
 * Browser state as reported by the managed host (Sections 2.4, 2.5).
 *
 * URL, title, loading state, security label and the tab set all come from the
 * runtime. A command the runtime refuses is not applied locally: the UI does not
 * show a URL the browser never reached and does not invent a tab the host never
 * opened. History walking, reload and tab operations are commands, never local
 * guesses about what the browser will do.
 */

export interface BrowserChromeState {
  /** Window identity, or `null` when no managed window is attached. */
  windowId: string | null;
  activeTabId: string | null;
  tabs: TabSummary[];
  /** Canonical address text as reported by the host. */
  address: string;
  title: string;
  origin: string;
  securityLabel: string;
  loading: boolean;
  private: boolean;
  /** Navigation availability, as reported by the host itself. */
  canGoBack: boolean;
  canGoForward: boolean;
}

export interface BrowserContextValue {
  state: BrowserChromeState;
  /** True only when a real host adapter supplied a validated channel. */
  bridgeAttached: boolean;
  hostLabel: string | null;
  /** Tabs the runtime reported, newest last. */
  tabs: TabSummary[];
  closeTab: (id: string) => void;
  activateTab: (id: string) => void;
  navigate: (url: string) => void;
  newTab: (url: string | null) => void;
  back: () => void;
  forward: () => void;
  reload: () => void;
  stop: () => void;
  focusAddress: () => void;
}

const BrowserContext = createContext<BrowserContextValue | null>(null);

const EMPTY_STATE: BrowserChromeState = {
  windowId: null,
  activeTabId: null,
  tabs: [],
  address: '',
  title: '',
  origin: '',
  securityLabel: '',
  loading: false,
  private: false,
  canGoBack: false,
  canGoForward: false,
};

export function BrowserProvider({ children }: { children: ReactNode }) {
  const runtime = useRuntime();

  const value = useMemo<BrowserContextValue>(() => {
    const view = runtime.snapshot.browser;
    const tabs: TabSummary[] = runtime.snapshot.tabs.map(([id, title]) => ({ id, title }));
    const state: BrowserChromeState =
      view === null
        ? { ...EMPTY_STATE, tabs, activeTabId: runtime.snapshot.activeTab }
        : {
            windowId: view.window,
            activeTabId: runtime.snapshot.activeTab ?? view.tab,
            tabs,
            address: view.address,
            title: view.title,
            origin: view.origin,
            securityLabel: view.security_label,
            loading: view.loading_state !== '',
            private: view.private,
            canGoBack: view.can_go_back,
            canGoForward: view.can_go_forward,
          };

    const send = (command: CommandBody): void => {
      runtime.client?.send(command);
    };

    return {
      state,
      bridgeAttached: runtime.runtimeConnected,
      hostLabel: runtime.hostLabel,
      tabs,
      closeTab: (id: string) => {
        send(commands.closeTab(id));
      },
      activateTab: (id: string) => {
        send(commands.activateTab(id));
      },
      navigate: (url: string) => {
        send(commands.navigate(url));
      },
      newTab: (url: string | null) => {
        send(commands.newTab(url));
      },
      back: () => {
        send(commands.back());
      },
      forward: () => {
        send(commands.forward());
      },
      reload: () => {
        send(commands.reload());
      },
      stop: () => {
        send(commands.stop());
      },
      focusAddress: () => {
        send(commands.focusAddressField());
      },
    };
  }, [runtime]);

  return <BrowserContext.Provider value={value}>{children}</BrowserContext.Provider>;
}

export function useBrowser(): BrowserContextValue {
  const ctx = useContext(BrowserContext);
  if (ctx === null) {
    throw new Error('useBrowser must be used within BrowserProvider');
  }
  return ctx;
}
