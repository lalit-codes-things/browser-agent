import React, { useContext, useEffect, useRef, useState } from 'react';
import { BrowserState, Tab, TabId, NativeBridge, BrowserApi } from './types';

function createBridgeApi(handle: number): NativeBridge {
  // The CEF host populates this once at startup. Until then every call is a
  // typed no-op so the UI can render and stay interactive (BA-BRIDGE-001).
  return {
    bridgeHandle: handle,
    get approvalState() {
      return null;
    },
    onEvent: () => () => undefined,
    navigate: () => Promise.resolve(),
    back: () => Promise.resolve(),
    forward: () => Promise.resolve(),
    reload: () => Promise.resolve(),
    stop: () => Promise.resolve(),
    focusAddress: () => Promise.resolve(),
    tabsCreate: () => Promise.resolve(''),
    tabsClose: () => Promise.resolve(),
    tabsActivate: () => Promise.resolve(),
    agentSubmitTask: () => Promise.resolve(''),
    agentPause: () => Promise.resolve(),
    agentCancel: () => Promise.resolve(),
    agentTakeOver: () => Promise.resolve(),
    agentResume: () => Promise.resolve(),
    agentRespondToApproval: () => Promise.resolve(),
  };
}

interface BrowserContextValue extends NativeBridge, BrowserApi {
  state: BrowserState;
}

const BrowserContext = React.createContext<BrowserContextValue | null>(null);

interface BrowserProviderProps {
  initial: BrowserState;
  onBridgeReady: (bridge: NativeBridge) => void;
  children: React.ReactNode;
}

export function BrowserProvider({ initial, onBridgeReady, children }: BrowserProviderProps) {
  const [state, setState] = useState<BrowserState>(initial);
  const handleRef = useRef(0);
  const apiRef = useRef<NativeBridge | null>(null);

  useEffect(() => {
    const api = createBridgeApi(handleRef.current++);
    apiRef.current = api;
    onBridgeReady(api);
  }, [onBridgeReady]);

  const value = React.useMemo(() => {
    const openTab = (url: string, title: string): Tab => {
      const tab: Tab = {
        id: crypto.randomUUID(),
        url,
        title,
        status: 'loading',
        lastUpdated: Date.now(),
        observationEpoch: crypto.randomUUID(),
      };
      setState((s) => ({
        ...s,
        tabs: [...s.tabs, tab],
        activeTabId: tab.id,
      }));
      return tab;
    };

    const closeTab = (id: TabId) => {
      setState((s) => {
        const tabs = s.tabs.filter((t) => t.id !== id);
        const nextId = tabs[tabs.length - 1]?.id ?? null;
        const activeId = nextId ?? s.activeTabId;
        return { ...s, tabs, activeTabId: activeId };
      });
    };

    const navigateTab = (id: TabId, url: string) => {
      const tab = state.tabs.find((t) => t.id === id);
      if (!tab) return;
      const currentUrl = tab.url;
      setState((s) => ({
        ...s,
        tabs: s.tabs.map((t) => (t.id === id ? { ...t, url, lastUpdated: Date.now() } : t)),
        currentUrl,
        activeTabId: id,
      }));
      void apiRef.current?.navigate(url);
    };

    const selectTab = (id: TabId) => {
      setState((s) => ({ ...s, activeTabId: id }));
    };

    const activateTab = (id: TabId) => {
      setState((s) => ({ ...s, activeTabId: id }));
    };

    return {
      bridgeHandle: handleRef.current,
      state,
      openTab,
      closeTab,
      navigateTab,
      selectTab,
      activateTab,
      async focusAddress() {
        await apiRef.current?.focusAddress();
      },
      async back() {
        await apiRef.current?.back();
      },
      async forward() {
        await apiRef.current?.forward();
      },
      async reload() {
        await apiRef.current?.reload();
      },
      async navigate(_url: string) {
        await apiRef.current?.navigate(_url);
      },
      async tabsCreate(_url: string, _title: string) {
        return apiRef.current?.tabsCreate(_url, _title) ?? '';
      },
      async tabsClose(_tabId: string) {
        await apiRef.current?.tabsClose(_tabId);
      },
      async tabsActivate(_tabId: string) {
        await apiRef.current?.tabsActivate(_tabId);
      },
      async agentSubmitTask(_taskText: string) {
        return apiRef.current?.agentSubmitTask(_taskText) ?? '';
      },
      async agentPause() {
        await apiRef.current?.agentPause();
      },
      async agentCancel() {
        await apiRef.current?.agentCancel();
      },
      async agentTakeOver() {
        await apiRef.current?.agentTakeOver();
      },
      async agentResume() {
        await apiRef.current?.agentResume();
      },
      async stop() {
        await apiRef.current?.stop();
      },
      async agentRespondToApproval(_actionId: string, _approved: boolean) {
        await apiRef.current?.agentRespondToApproval(_actionId, _approved);
      },
      get approvalState() {
        return apiRef.current?.approvalState ?? null;
      },
      onEvent: () => () => undefined,
    };
  }, [state]);

  const { Provider } = BrowserContext;
  return React.createElement(Provider, { value }, children);
}

export function useBrowser(): BrowserContextValue {
  const ctx = useContext(BrowserContext);
  if (!ctx) throw new Error('useBrowser must be used within BrowserProvider');
  return ctx;
}
