import { useRef, useCallback, useMemo } from 'react';
import type { Tab, TabId, NativeBridge } from './types';

export interface TabsApi {
  activeTabId: TabId | null;
  open: (url: string, title: string) => Tab;
  close: (id: TabId) => void;
  navigate: (id: TabId, url: string) => void;
  activate: (id: TabId) => void;
  onBridgeReady: (api: NativeBridge) => void;
  bridgeHandle: number;
}

export function useTabs(): TabsApi {
  const browserRef = useRef<{ activeTabId: TabId | null } | null>(null);
  const apiRef = useRef<NativeBridge | null>(null);
  const tabsRef = useRef<Tab[]>([]);

  const remove = useCallback(
    (id: TabId) => {
      const index = tabsRef.current.findIndex((t) => t.id === id);
      if (index === -1) return;
      tabsRef.current = tabsRef.current.filter((t) => t.id !== id);
      if (browserRef.current?.activeTabId === id) {
        browserRef.current.activeTabId = tabsRef.current[tabsRef.current.length - 1]?.id ?? null;
      }
      void apiRef.current?.tabsClose?.(id);
    },
    [browserRef, apiRef, tabsRef],
  );

  const navigate = useCallback(
    (id: TabId, url: string) => {
      const tab = tabsRef.current.find((t) => t.id === id);
      if (!tab) return;
      tab.url = url;
      tab.lastUpdated = Date.now();
      if (browserRef.current?.activeTabId === id) {
        browserRef.current.activeTabId = id;
      }
      void apiRef.current?.navigate?.(url);
    },
    [browserRef, apiRef, tabsRef],
  );

  const activate = useCallback(
    (id: TabId) => {
      if (browserRef.current) {
        browserRef.current.activeTabId = id;
      }
      void apiRef.current?.tabsActivate?.(id);
      tabsRef.current = tabsRef.current.map((t) =>
        t.id === id ? { ...t, lastUpdated: Date.now() } : t,
      );
    },
    [browserRef, apiRef, tabsRef],
  );

  const open = useCallback((url: string, title: string): Tab => {
    const tab: Tab = {
      id: crypto.randomUUID(),
      url,
      title,
      status: 'loading',
      lastUpdated: Date.now(),
      observationEpoch: crypto.randomUUID(),
    };
    tabsRef.current = [...tabsRef.current, tab];
    if (browserRef.current) {
      browserRef.current.activeTabId = tab.id;
    }
    void apiRef.current?.tabsCreate?.(url, title);
    return tab;
  }, []);

  const lastTab = tabsRef.current.at(-1);
  const activeTabId = lastTab?.id ?? null;

  const bridgeHandle = apiRef.current?.bridgeHandle ?? 0;

  const onBridgeReady = useCallback((api: NativeBridge) => {
    apiRef.current = api;
  }, []);

  return useMemo(
    () => ({
      activeTabId,
      open,
      close: remove,
      navigate,
      activate,
      onBridgeReady,
      bridgeHandle,
    }),
    [activeTabId, bridgeHandle, open, remove, navigate, activate, onBridgeReady],
  );
}
