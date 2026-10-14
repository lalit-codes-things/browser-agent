import { useCallback, useEffect, useRef, useState } from 'react';
import type { BrowserApi } from '../domain/types';

/**
 * Browser chrome.
 *
 * Icon whitelist (Section 2.6): Undo, Redo, Reload/Stop, Bookmarks, Downloads and
 * the three-dot overflow menu. Nothing else may ship as an app-owned icon, so
 * Back and Forward are text-labelled controls ([BA-LAYOUT-003]) and every other
 * command is a labelled menu item or panel control.
 *
 * A control ships only when its operation exists and can report a real result.
 * Undo and Redo render disabled with an explanation because no app-owned
 * operation currently has a recorded precondition plus a tested compensator
 * (Section 2.7). Bookmarks and Downloads are omitted entirely: their runtime
 * operations do not exist yet, and a control without real behavior may not ship.
 *
 * No ripple, glow, gradient or decorative animation is used anywhere here.
 */

export const OVERFLOW_COMMANDS = [{ id: 'about', label: 'About Browser-Agent' }] as const;

export type OverflowCommandId = (typeof OVERFLOW_COMMANDS)[number]['id'];

interface Props {
  browser: BrowserApi;
  onNavigate: (url: string) => void;
  onCommand: (command: OverflowCommandId) => void;
}

export function Toolbar({ browser, onNavigate, onCommand }: Props) {
  const runtimeUrl = browser.state.currentUrl;
  const [address, setAddress] = useState(runtimeUrl);

  // The runtime owns the current URL; the field never keeps a URL the runtime
  // did not report.
  useEffect(() => {
    setAddress(runtimeUrl);
  }, [runtimeUrl]);

  const submitAddress = useCallback(() => {
    const trimmed = address.trim();
    if (trimmed.length === 0) {
      setAddress(runtimeUrl);
      return;
    }
    onNavigate(trimmed);
  }, [address, onNavigate, runtimeUrl]);

  return (
    <div className="ba-toolbar">
      <button
        type="button"
        className="ba-toolbar-text"
        onClick={() => void browser.back()}
        disabled={!browser.state.canGoBack}
      >
        Back
      </button>
      <button
        type="button"
        className="ba-toolbar-text"
        onClick={() => void browser.forward()}
        disabled={!browser.state.canGoForward}
      >
        Forward
      </button>

      <label className="ba-toolbar__address">
        <span className="ba-visually-hidden">Address</span>
        <input
          className="ba-toolbar-address-input"
          value={address}
          onChange={(event) => setAddress(event.target.value)}
          onBlur={submitAddress}
          onKeyDown={(event) => {
            if (event.key === 'Enter') {
              event.preventDefault();
              submitAddress();
            }
          }}
          placeholder="Enter a URL"
          spellCheck={false}
        />
      </label>

      <div className="ba-toolbar__items">
        <ToolbarIconControl
          label="Undo"
          modifier="undo"
          disabled
          explanation="No app-owned operation has a tested compensating action yet."
        />
        <ToolbarIconControl
          label="Redo"
          modifier="redo"
          disabled
          explanation="Nothing has been compensated, so there is nothing to redo."
        />
        <ToolbarIconControl
          label="Reload"
          modifier="reload"
          onClick={() => void browser.reload()}
        />
        <OverflowMenu onCommand={onCommand} />
      </div>
    </div>
  );
}

function ToolbarIconControl({
  label,
  modifier,
  onClick,
  disabled = false,
  explanation,
}: {
  label: string;
  modifier: 'undo' | 'redo' | 'reload';
  onClick?: (() => void) | undefined;
  disabled?: boolean;
  explanation?: string | undefined;
}) {
  return (
    <button
      type="button"
      className={`ba-toolbar-icon-control ba-toolbar-icon-control--${modifier}`}
      aria-label={label}
      title={explanation ?? label}
      onClick={onClick}
      disabled={disabled}
    >
      <span className="ba-toolbar-icon" aria-hidden="true" />
    </button>
  );
}

function OverflowMenu({ onCommand }: { onCommand: (command: OverflowCommandId) => void }) {
  const [open, setOpen] = useState(false);
  const triggerRef = useRef<HTMLButtonElement | null>(null);

  const close = useCallback((returnFocus: boolean) => {
    setOpen(false);
    if (returnFocus) triggerRef.current?.focus();
  }, []);

  return (
    <div
      className="ba-overflow"
      onKeyDown={(event) => {
        if (event.key === 'Escape' && open) {
          event.preventDefault();
          close(true);
        }
      }}
    >
      <button
        ref={triggerRef}
        type="button"
        className="ba-toolbar-icon-control ba-toolbar-icon-control--overflow"
        aria-label="Menu"
        title="Menu"
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen((value) => !value)}
      >
        <span className="ba-toolbar-icon" aria-hidden="true" />
      </button>

      {open && (
        <ul className="ba-overflow-menu" role="menu" aria-label="Browser commands">
          {OVERFLOW_COMMANDS.map((command) => (
            <li key={command.id} role="none">
              <button
                type="button"
                role="menuitem"
                className="ba-overflow-item"
                onClick={() => {
                  onCommand(command.id);
                  close(true);
                }}
              >
                {command.label}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
