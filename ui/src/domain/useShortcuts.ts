import { useEffect } from 'react';
import { useBrowser } from './useBrowser';
import { useAgent } from './useAgent';

/**
 * Core shortcut baseline (Section 18.2).
 *
 * The table below is the single source for both behaviour and the Settings
 * documentation, so the interface cannot document a shortcut it does not
 * implement. Only operations that exist are wired: reopening a closed tab has no
 * runtime operation in this build, so it appears in the "not implemented" list
 * instead of as a working hint.
 *
 * Shortcuts are never stolen from text entry for actions that would destroy the
 * user's text: `Escape` inside a field stays with the field, while navigation
 * and tab commands behave as they do in a normal browser.
 */

export type ShortcutId =
  'focusAddress' | 'newTab' | 'closeTab' | 'reload' | 'nextTab' | 'previousTab' | 'stop';

export interface ShortcutDefinition {
  id: ShortcutId;
  /** What the shortcut does, in user language. */
  description: string;
  /** Key names in text form; macOS command glyphs are never printed. */
  mac: string;
  windows: string;
}

/** Implemented shortcuts, in the documented order. */
export const SHORTCUTS: readonly ShortcutDefinition[] = [
  {
    id: 'focusAddress',
    description: 'Focus the address bar',
    mac: 'Command + L',
    windows: 'Control + L',
  },
  { id: 'newTab', description: 'New tab', mac: 'Command + T', windows: 'Control + T' },
  { id: 'closeTab', description: 'Close tab', mac: 'Command + W', windows: 'Control + W' },
  { id: 'reload', description: 'Reload', mac: 'Command + R', windows: 'Control + R' },
  { id: 'nextTab', description: 'Next tab', mac: 'Control + Tab', windows: 'Control + Tab' },
  {
    id: 'previousTab',
    description: 'Previous tab',
    mac: 'Control + Shift + Tab',
    windows: 'Control + Shift + Tab',
  },
  {
    id: 'stop',
    description: 'Stop loading the current page',
    mac: 'Escape',
    windows: 'Escape',
  },
];

/**
 * Baseline shortcuts this build does not implement, with the reason. Stated
 * explicitly so the absence is visible rather than silently missing.
 */
export const UNIMPLEMENTED_SHORTCUTS: readonly { label: string; reason: string }[] = [
  {
    label: 'Reopen closed tab',
    reason:
      'The runtime keeps no closed-tab record yet, so reopening one would invent a tab the host never restored.',
  },
];

/** True on a macOS host, where the platform modifier is Command. */
export function isMacPlatform(): boolean {
  if (typeof navigator === 'undefined') return false;
  const source = navigator.userAgent;
  return source.includes('Macintosh') || source.includes('Mac OS');
}

/** Platform-correct shortcut text for one definition. */
export function shortcutText(definition: ShortcutDefinition): string {
  return isMacPlatform() ? definition.mac : definition.windows;
}

function isTextEntry(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  return tag === 'INPUT' || tag === 'TEXTAREA' || target.isContentEditable;
}

/** Wires the core shortcut baseline to the runtime commands. */
export function useShortcuts(): void {
  const browser = useBrowser();
  const agent = useAgent();

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent): void => {
      const commandModifier = isMacPlatform() ? event.metaKey : event.ctrlKey;
      const inTextEntry = isTextEntry(event.target);

      // Escape inside a text field belongs to the field.
      if (event.key === 'Escape' && !inTextEntry && browser.state.loading) {
        event.preventDefault();
        browser.stop();
        return;
      }

      if (event.key === 'Tab' && event.ctrlKey) {
        // Control + Tab is a navigation command in every supported browser, so
        // it applies while typing too.
        const tabs = browser.tabs;
        if (tabs.length < 2) return;
        const index = tabs.findIndex((tab) => tab.id === browser.state.activeTabId);
        if (index === -1) return;
        const nextIndex = event.shiftKey
          ? (index - 1 + tabs.length) % tabs.length
          : (index + 1) % tabs.length;
        const next = tabs[nextIndex];
        if (next === undefined) return;
        event.preventDefault();
        browser.activateTab(next.id);
        return;
      }

      if (!commandModifier) return;

      switch (event.key.toLowerCase()) {
        case 'l':
          event.preventDefault();
          browser.focusAddress();
          break;
        case 't':
          event.preventDefault();
          browser.newTab(null);
          break;
        case 'w': {
          const activeTab = browser.state.activeTabId;
          if (activeTab === null) return;
          event.preventDefault();
          browser.closeTab(activeTab);
          break;
        }
        case 'r':
          event.preventDefault();
          browser.reload();
          break;
        default:
          break;
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [agent, browser]);
}
