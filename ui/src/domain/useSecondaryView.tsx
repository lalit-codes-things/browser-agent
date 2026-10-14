import { createContext, useContext, useState, ReactNode } from 'react';
import { SecondaryView } from './types';

interface SecondaryViewContextValue {
  active: SecondaryView | null;
  open: (name: SecondaryView) => void;
  close: () => void;
}

const SecondaryViewContext = createContext<SecondaryViewContextValue | null>(null);

export function SecondaryViewProvider({ children }: { children: ReactNode }) {
  const [active, setActive] = useState<SecondaryView | null>(null);

  return (
    <SecondaryViewContext.Provider
      value={{ active, open: setActive, close: () => setActive(null) }}
    >
      {children}
      {active !== null && <SecondaryViewScreen name={active} onClose={() => setActive(null)} />}
    </SecondaryViewContext.Provider>
  );
}

function SecondaryViewScreen({ name, onClose }: { name: SecondaryView; onClose: () => void }) {
  return (
    <div className="ba-secondary-backdrop" onClick={onClose}>
      <div className="ba-secondary-sheet" role="dialog" aria-modal="true" aria-label={name}>
        <h2 className="ba-secondary-heading">{name}</h2>
        <p className="ba-secondary-body">
          Implementation of this screen is out of scope for the initial build.
        </p>
        <button className="ba-btn ba-btn-secondary" onClick={onClose}>
          Close
        </button>
      </div>
    </div>
  );
}

export function useSecondaryView() {
  const ctx = useContext(SecondaryViewContext);
  if (!ctx) throw new Error('useSecondaryView must be used within SecondaryViewProvider');
  return ctx;
}
