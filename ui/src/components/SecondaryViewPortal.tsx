import { useEffect } from 'react';
import type { ReactNode } from 'react';

/**
 * Secondary screens are opened from real menus and render real content. Content
 * is required: a sheet with placeholder copy would violate the no-dead-control
 * rule ([BA-2.8]).
 */

interface Props {
  name: string;
  onClose: () => void;
  children: ReactNode;
}

export function SecondaryViewPortal({ name, onClose, children }: Props) {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [onClose]);

  return (
    <div className="ba-secondary-backdrop">
      <div className="ba-secondary-sheet" role="dialog" aria-modal="true" aria-label={name}>
        <h2 className="ba-secondary-heading">{name}</h2>
        {children}
        <div className="ba-secondary-actions">
          <button type="button" className="ba-btn ba-btn--secondary" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
