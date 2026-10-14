import { Tab, TabId } from '../domain';

interface Props {
  tabs: readonly Tab[];
  activeTabId: TabId | null;
  onSelect: (id: TabId) => void;
  onClose: (id: TabId) => void;
}

function statusLabel(tab: Tab): string {
  switch (tab.status) {
    case 'loading':
      return 'Loading';
    case 'ready':
      return tab.title.length > 36 ? tab.title.slice(0, 33) + '…' : tab.title;
    case 'blocked':
      return 'Blocked';
    case 'error':
      return 'Error';
    default:
      return tab.title.length > 36 ? tab.title.slice(0, 33) + '…' : tab.title;
  }
}

export function TabStrip({ tabs, activeTabId, onSelect, onClose }: Props) {
  return (
    <div className="ba-tabstrip" role="tablist" aria-label="Tabs">
      {tabs.map((tab) => (
        <div
          key={tab.id}
          role="tab"
          aria-selected={activeTabId === tab.id}
          className={`ba-tab ${activeTabId === tab.id ? 'ba-tab--active' : ''}`}
          onClick={() => onSelect(tab.id)}
        >
          <span className="ba-tab-title" title={tab.title}>
            {statusLabel(tab)}
          </span>
          <button
            className="ba-tab-close"
            aria-label={`Close tab ${tab.title}`}
            onClick={(e) => {
              e.stopPropagation();
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
