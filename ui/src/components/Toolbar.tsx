import { useEffect, useRef, useState, useCallback } from 'react';
import type { BrowserApi } from '../domain/types';

interface Props {
  browser: BrowserApi;
  onNavigate: (url: string, appId?: string) => void;
}

function useToolbarRipple() {
  const poolRef = useRef<HTMLSpanElement[]>([]);

  useEffect(() => {
    const rafPool = new Set<Element>();
    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        for (const removed of mutation.removedNodes) {
          if (removed instanceof HTMLElement) {
            rafPool.add(removed);
            for (const child of removed.querySelectorAll?.('.ba-toolbar-ripple') ?? []) {
              rafPool.add(child);
            }
          }
        }
      }
      if (rafPool.size > 0) {
        requestAnimationFrame(() => {
          for (const node of rafPool) {
            if (node instanceof HTMLElement) {
              node.remove();
            }
          }
          rafPool.clear();
        });
      }
    });

    for (const ripple of poolRef.current) {
      observer.observe(ripple, {
        childList: true,
        subtree: true,
      });
    }

    return () => {
      observer.disconnect();
      for (const ripple of poolRef.current) {
        ripple.remove();
      }
      poolRef.current = [];
    };
  }, []);

  const start = useCallback(
    (target: HTMLElement, clientX: number, clientY: number, keyboard = false) => {
      const rect = target.getBoundingClientRect();
      const ripple = poolRef.current.pop() ?? document.createElement('span');
      ripple.className = 'ba-toolbar-ripple';
      ripple.style.pointerEvents = 'none';
      ripple.style.position = 'absolute';
      if (keyboard) {
        ripple.style.left = '0px';
        ripple.style.top = '0px';
        Object.assign(ripple.dataset, { keyboard: 'true' });
      } else {
        ripple.style.left = `${clientX - rect.left}px`;
        ripple.style.top = `${clientY - rect.top}px`;
      }
      Object.assign(target.style, { position: 'relative' });
      target.appendChild(ripple);
      poolRef.current.push(ripple);
    },
    [],
  );

  return { start };
}

function ToolbarActionButton({
  className = '',
  'aria-label': ariaLabel,
  title,
  onClick,
  children,
  ripple,
}: {
  className?: string;
  'aria-label': string;
  title?: string;
  onClick: () => void;
  children: React.ReactNode;
  ripple: ReturnType<typeof useToolbarRipple>['start'];
}) {
  const buttonRef = useRef<HTMLButtonElement | null>(null);

  useEffect(() => {
    const button = buttonRef.current;
    if (button == null) return;

    const handlePointerDown = (event: PointerEvent) => {
      if (event.button !== 0) return;
      if (button.disabled || button.getAttribute('aria-disabled') === 'true') {
        return;
      }
      ripple(button, event.clientX, event.clientY);
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== 'Enter' && event.key !== 'Space') return;
      if (button.disabled || button.getAttribute('aria-disabled') === 'true') {
        return;
      }
      ripple(button, 0, 0, true);
      event.preventDefault();
    };

    button.addEventListener('pointerdown', handlePointerDown);
    button.addEventListener('keydown', handleKeyDown);
    return () => {
      button.removeEventListener('pointerdown', handlePointerDown);
      button.removeEventListener('keydown', handleKeyDown);
    };
  }, [ripple]);

  return (
    <button
      ref={buttonRef}
      className={'ba-toolbar-item ba-toolbar-item--interactive ' + className}
      aria-label={ariaLabel}
      title={title}
      onClick={onClick}
    >
      {children}
    </button>
  );
}

export function Toolbar({ browser, onNavigate }: Props) {
  const [address, setAddress] = useState(browser.state.currentUrl ?? '');
  const inputRef = useRef<HTMLInputElement | null>(null);
  const { start: ripple } = useToolbarRipple();

  const focusAddress = useCallback(() => {
    inputRef.current?.focus();
    void browser.focusAddress();
  }, [browser]);

  const submitFromAddress = useCallback(() => {
    const trimmed = address.trim();
    if (!trimmed) return;
    if (trimmed.startsWith('browser-agent://')) {
      onNavigate(trimmed);
      return;
    }
    const lastSegment = trimmed.split('/').slice(-1)[0];
    onNavigate(trimmed, lastSegment ?? trimmed);
  }, [address, onNavigate]);

  return (
    <div className="ba-toolbar">
      <div className="ba-toolbar__address">
        <button
          className="ba-toolbar-item ba-toolbar-item--icon"
          aria-label="Focus address"
          title="Focus address bar"
          onClick={focusAddress}
        >
          <span className="ba-toolbar-icon ba-toolbar-icon--url" />
        </button>
        <input
          ref={inputRef}
          className="ba-toolbar-address-input"
          value={address}
          onChange={(e) => setAddress(e.target.value)}
          onBlur={submitFromAddress}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              submitFromAddress();
            }
          }}
          placeholder="Enter URL or paste a task"
          spellCheck={false}
        />
      </div>

      <div className="ba-toolbar__items">
        <ToolbarActionButton
          className="ba-toolbar-item--back"
          aria-label="Back"
          onClick={() => void browser.back()}
          ripple={ripple}
        >
          <span className="ba-toolbar-icon ba-toolbar-icon--back" />
        </ToolbarActionButton>
        <ToolbarActionButton
          className="ba-toolbar-item--forward"
          aria-label="Forward"
          onClick={() => void browser.forward()}
          ripple={ripple}
        >
          <span className="ba-toolbar-icon ba-toolbar-icon--forward" />
        </ToolbarActionButton>
        <ToolbarActionButton
          className="ba-toolbar-item--icon"
          aria-label="Reload"
          title="Reload page"
          onClick={() => {
            void browser.reload();
          }}
          ripple={ripple}
        >
          <span className="ba-toolbar-icon ba-toolbar-icon--reload" />
        </ToolbarActionButton>
        <ToolbarActionButton
          className="ba-toolbar-item--icon"
          aria-label="Undo"
          title="Undo"
          onClick={() => undefined}
          ripple={ripple}
        >
          <span className="ba-toolbar-icon ba-toolbar-icon--undo" />
        </ToolbarActionButton>
        <ToolbarActionButton
          className="ba-toolbar-item--icon"
          aria-label="Redo"
          title="Redo"
          onClick={() => undefined}
          ripple={ripple}
        >
          <span className="ba-toolbar-icon ba-toolbar-icon--redo" />
        </ToolbarActionButton>
        <ToolbarActionButton
          className="ba-toolbar-item--icon"
          aria-label="Bookmarks"
          title="Bookmarks"
          onClick={() => undefined}
          ripple={ripple}
        >
          <span className="ba-toolbar-icon ba-toolbar-icon--bookmark" />
        </ToolbarActionButton>
        <ToolbarActionButton
          className="ba-toolbar-item--icon"
          aria-label="Downloads"
          title="Downloads"
          onClick={() => undefined}
          ripple={ripple}
        >
          <span className="ba-toolbar-icon ba-toolbar-icon--download" />
        </ToolbarActionButton>
      </div>
    </div>
  );
}
