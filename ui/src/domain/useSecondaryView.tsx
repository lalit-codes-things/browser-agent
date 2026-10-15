import { createContext, useCallback, useContext, useMemo, useState } from 'react';
import type { ReactNode } from 'react';
import type { SecondaryView } from './types';

/**
 * Which optional secondary screen is open (Sections 2.8, 2.13).
 *
 * The provider holds only that selection; the owning shell renders the screen
 * with real runtime content. A screen may not ship placeholder copy, so only
 * screens backed by real runtime operations are listed here.
 */

export type SecondaryViewName = SecondaryView;

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
