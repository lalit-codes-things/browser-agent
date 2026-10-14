import { ReactNode } from 'react';
import { SecondaryView } from '../domain';

interface Props {
  name: SecondaryView;
  onClose: () => void;
  children?: ReactNode;
}

/**
 * Secondary screens are rendered from real menus/links and lazy-load on demand.
 * No dead menu item or decorative placeholder may ship (BA-2.8).
 */
export function SecondaryViewPortal({ name, onClose, children }: Props) {
  return (
    <div className="ba-secondary-backdrop" onClick={onClose}>
      <div className="ba-secondary-sheet" role="dialog" aria-modal="true" aria-label={name}>
        <h2 className="ba-secondary-heading">{name}</h2>
        {children ?? (
          <p className="ba-secondary-body">
            Implementation of this screen is out of scope for the initial build.
          </p>
        )}
        <button className="ba-btn ba-btn-secondary" onClick={onClose}>
          Close
        </button>
      </div>
    </div>
  );
}
