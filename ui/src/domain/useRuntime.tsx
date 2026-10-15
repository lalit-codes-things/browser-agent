import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  useSyncExternalStore,
} from 'react';
import type { ReactNode } from 'react';
import { attachHostChannel } from '../bridge/host';
import type { HostChannel, RuntimeClient, RuntimeSnapshot } from '../bridge/client';
import { createRuntimeClient } from '../bridge/client';

/**
 * The runtime context owns the single connection to the Rust application core.
 *
 * The host adapter is asked once for its channel. When it attaches, the UI
 * renders authoritative runtime state; when it does not, the snapshot stays
 * empty and every consumer reports that the runtime is not connected. No
 * component receives a stub client that could be mistaken for a live runtime.
 */

/** Stable empty snapshot used while no host is attached. */
const ABSENT_SNAPSHOT: RuntimeSnapshot = {
  attached: false,
  browser: null,
  tabs: [],
  activeTab: null,
  readiness: null,
  task: null,
  activity: [],
  ended: null,
  approval: null,
  handoff: null,
  history: null,
  notes: null,
  skills: null,
  downloads: null,
  audit: null,
  error: null,
  lastRefusal: null,
  refusals: 0,
};

export interface RuntimeContextValue {
  /** The runtime client, or `null` when no host has attached. */
  client: RuntimeClient | null;
  snapshot: RuntimeSnapshot;
  /** True only when a real host adapter supplied a validated channel. */
  runtimeConnected: boolean;
  /** Host identity for diagnostics; never a capability. */
  hostLabel: string | null;
}

const RuntimeContext = createContext<RuntimeContextValue | null>(null);

interface RuntimeProviderProps {
  children: ReactNode;
  /**
   * Explicit channel, used by the shell when the host adapter hands one over
   * itself. When omitted the host property is read once at mount.
   */
  channel?: HostChannel | null | undefined;
}

export function RuntimeProvider({ children, channel }: RuntimeProviderProps) {
  const [client] = useState<RuntimeClient | null>(() => {
    const resolved = channel ?? attachHostChannel();
    return resolved === null ? null : createRuntimeClient(resolved);
  });

  const snapshot = useSyncExternalStore(
    useCallback(
      (listener: () => void) => client?.subscribe(listener) ?? ((): void => undefined),
      [client],
    ),
    useCallback(() => client?.snapshot() ?? ABSENT_SNAPSHOT, [client]),
  );

  const value = useMemo<RuntimeContextValue>(
    () => ({
      client,
      snapshot,
      runtimeConnected: client !== null && snapshot.attached,
      hostLabel: client?.hostLabel ?? null,
    }),
    [client, snapshot],
  );

  return <RuntimeContext.Provider value={value}>{children}</RuntimeContext.Provider>;
}

export function useRuntime(): RuntimeContextValue {
  const ctx = useContext(RuntimeContext);
  if (ctx === null) {
    throw new Error('useRuntime must be used within RuntimeProvider');
  }
  return ctx;
}
