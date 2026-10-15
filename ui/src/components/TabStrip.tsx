import type { ReactNode } from 'react';
import type { TabSummary } from '../domain/types';

/**
 * Tab strip (Section 2.4, [BA-TABSTATE-001]).
 *
 * The host owns the tab set and each title; the strip renders exactly what the
 * host reported. A tab's title is never replaced by a status label, and no
 * app-owned icon is added beyond the permitted close control.
 */

interface Props {
  tabs: readonly TabSummary[];
  activeTabId: string | null;
  onSelect: (id: string) => void;
  onClose: (id: string) => void;
  /** Rendered instead of tabs when the runtime reports none. */
  emptyState?: ReactNode;
}

export function TabStrip({ tabs, activeTabId, onSelect, onClose, emptyState }: Props) {
  return (
    <div className="ba-tabstrip" role="tablist" aria-label="Tabs">
      {tabs.length === 0 && emptyState}
      {tabs.map((tab) => (
        <div
          key={tab.id}
          role="tab"
          tabIndex={0}
          aria-selected={activeTabId === tab.id}
          className={`ba-tab${activeTabId === tab.id ? ' ba-tab--active' : ''}`}
          onClick={() => onSelect(tab.id)}
          onKeyDown={(event) => {
            if (event.key === 'Enter' || event.key === ' ') {
              event.preventDefault();
              onSelect(tab.id);
            }
          }}
        >
          <span className="ba-tab-title" title={tab.title}>
            {tab.title}
          </span>
          <button
            type="button"
            className="ba-tab-close"
            aria-label={`Close tab ${tab.title}`}
            onClick={(event) => {
              event.stopPropagation();
              onClose(tab.id);
            }}
          >
            ×
          </button>
        </div>
      ))}
    </div>
  );
}
