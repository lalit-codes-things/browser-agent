import { createContext, useCallback, useContext, useMemo, useState } from 'react';
import type { ReactNode } from 'react';

/**
 * State for optional secondary screens (Settings, History, About, Downloads).
 * The provider holds only which screen is open; the owning shell renders it
 * through `SecondaryViewPortal` with real content. No secondary screen may ship
 * placeholder copy ([BA-2.8]).
 */

export type SecondaryViewName = 'about';

interface SecondaryViewContextValue {
  active: SecondaryViewName | null;
  open: (name: SecondaryViewName) => void;
  close: () => void;
}

const SecondaryViewContext = createContext<SecondaryViewContextValue | null>(null);

export function SecondaryViewProvider({ children }: { children: ReactNode }) {
  const [active, setActive] = useState<SecondaryViewName | null>(null);

  const open = useCallback((name: SecondaryViewName) => setActive(name), []);
  const close = useCallback(() => setActive(null), []);

  const value = useMemo(() => ({ active, open, close }), [active, open, close]);

  return <SecondaryViewContext.Provider value={value}>{children}</SecondaryViewContext.Provider>;
}

export function useSecondaryView(): SecondaryViewContextValue {
  const ctx = useContext(SecondaryViewContext);
  if (ctx === null) {
    throw new Error('useSecondaryView must be used within SecondaryViewProvider');
  }
  return ctx;
}
