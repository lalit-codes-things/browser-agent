import type { ReactNode } from 'react';
import type { Tab, TabId } from '../domain/types';

/**
 * Tab strip. A tab's page title is never replaced by a status label; the status
 * is a textual suffix on the same control ([BA-TABSTATE-001]). Loading, crash
 * and audio states use text only: no extra app-owned icon and no ornamental
 * spinner animation.
 */

interface Props {
  tabs: readonly Tab[];
  activeTabId: TabId | null;
  onSelect: (id: TabId) => void;
  onClose: (id: TabId) => void;
  /** Rendered instead of tabs when the runtime reports none. */
  emptyState?: ReactNode;
}

const STATUS_SUFFIX: Record<Tab['status'], string> = {
  loading: 'Loading',
  ready: '',
  blocked: 'Blocked',
  error: 'Error',
};

function tabLabel(tab: Tab): string {
  const suffix = STATUS_SUFFIX[tab.status];
  return suffix === '' ? tab.title : `${tab.title} — ${suffix}`;
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
            {tabLabel(tab)}
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
