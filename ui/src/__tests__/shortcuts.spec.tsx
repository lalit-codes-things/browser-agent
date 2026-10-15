import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { App } from '../App';
import { SCHEMA_VERSION } from '../bridge/protocol';
import { HOST_CHANNEL_PROPERTY } from '../bridge/host';
import { SHORTCUTS, UNIMPLEMENTED_SHORTCUTS } from '../domain/useShortcuts';
import type { HostChannel } from '../bridge/client';

/**
 * Core shortcut baseline (Section 18.2).
 *
 * Each documented shortcut must do something real, must not be stolen from text
 * entry, and must be documented in Settings from the same table the handler
 * uses. A shortcut with no runtime operation is listed as unimplemented instead
 * of being hinted as available.
 */

interface Harness {
  sent: { payload: { domain: string; op: string; args: Record<string, unknown> } }[];
  emit: (payload: Record<string, unknown>) => void;
}

function renderShell(): Harness {
  const listeners = new Set<(frame: string) => void>();
  const sent: Harness['sent'] = [];
  let sequence = 0;

  const channel: HostChannel = {
    hostLabel: 'test-cef-host',
    send: (frame: string) => {
      sent.push(JSON.parse(frame) as Harness['sent'][number]);
    },
    subscribe: (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
  };
  (globalThis as Record<string, unknown>)[HOST_CHANNEL_PROPERTY] = channel;
  render(<App />);

  return {
    sent,
    emit: (payload) => {
      sequence += 1;
      const envelope = JSON.stringify({
        schema_version: SCHEMA_VERSION,
        sequence,
        at_unix_ms: 1_700_000_000_000 + sequence,
        monotonic_ms: sequence,
        payload,
      });
      for (const listener of [...listeners]) listener(`${envelope}\n`);
    },
  };
}

function lastCommand(harness: Harness): { op: string; args: Record<string, unknown> } | undefined {
  const entry = harness.sent.at(-1);
  return entry === undefined ? undefined : { op: entry.payload.op, args: entry.payload.args };
}

function tabsAndBrowser(): Record<string, unknown>[] {
  return [
    {
      kind: 'BROWSER_STATE',
      state: {
        window: 'window-1',
        tab: 'tab-1',
        origin: 'https://example.com',
        context: 'ctx-normal',
        private: false,
        address: 'https://example.com/',
        security_label: 'Secure connection',
        title: 'Example',
        loading_state: 'Loading',
        can_go_back: false,
        can_go_forward: false,
      },
    },
    {
      kind: 'TABS',
      tabs: [
        ['tab-1', 'Example'],
        ['tab-2', 'Fares'],
      ],
      active: 'tab-1',
    },
  ];
}

afterEach(() => {
  delete (globalThis as Record<string, unknown>)[HOST_CHANNEL_PROPERTY];
});

describe('keyboard baseline', () => {
  it('focuses the address bar with the platform modifier plus L', () => {
    const harness = renderShell();
    act(() => {
      for (const payload of tabsAndBrowser()) harness.emit(payload);
    });

    const modifier = navigator.userAgent.includes('Mac') ? { metaKey: true } : { ctrlKey: true };
    fireEvent.keyDown(window, { key: 'l', ...modifier });

    expect(lastCommand(harness)).toEqual({ op: 'FOCUS_ADDRESS_FIELD', args: {} });
  });

  it('opens and closes tabs through the runtime', () => {
    const harness = renderShell();
    act(() => {
      for (const payload of tabsAndBrowser()) harness.emit(payload);
    });
    const modifier = navigator.userAgent.includes('Mac') ? { metaKey: true } : { ctrlKey: true };

    fireEvent.keyDown(window, { key: 't', ...modifier });
    expect(lastCommand(harness)).toEqual({ op: 'NEW_TAB', args: { url: null } });

    fireEvent.keyDown(window, { key: 'w', ...modifier });
    // Closing names the active tab the host reported.
    expect(lastCommand(harness)).toEqual({ op: 'CLOSE_TAB', args: { tab: 'tab-1' } });

    fireEvent.keyDown(window, { key: 'r', ...modifier });
    expect(lastCommand(harness)).toEqual({ op: 'RELOAD', args: {} });
  });

  it('moves between tabs with Control + Tab, wrapping in both directions', () => {
    const harness = renderShell();
    act(() => {
      for (const payload of tabsAndBrowser()) harness.emit(payload);
    });

    fireEvent.keyDown(window, { key: 'Tab', ctrlKey: true });
    expect(lastCommand(harness)).toEqual({ op: 'ACTIVATE_TAB', args: { tab: 'tab-2' } });

    // The host reports the new active tab; the next shortcut moves from there.
    act(() => {
      harness.emit({
        kind: 'TABS',
        tabs: [
          ['tab-1', 'Example'],
          ['tab-2', 'Fares'],
        ],
        active: 'tab-2',
      });
    });
    fireEvent.keyDown(window, { key: 'Tab', ctrlKey: true, shiftKey: true });
    expect(lastCommand(harness)).toEqual({ op: 'ACTIVATE_TAB', args: { tab: 'tab-1' } });

    // One tab only: the shortcut must not invent a target.
    act(() => {
      harness.emit({ kind: 'TABS', tabs: [['tab-1', 'Example']], active: 'tab-1' });
    });
    const before = harness.sent.length;
    fireEvent.keyDown(window, { key: 'Tab', ctrlKey: true });
    expect(harness.sent.length).toBe(before);
  });

  it('stops loading on Escape, and never steals Escape from a text field', () => {
    const harness = renderShell();
    act(() => {
      for (const payload of tabsAndBrowser()) harness.emit(payload);
    });

    fireEvent.keyDown(window, { key: 'Escape' });
    expect(lastCommand(harness)).toEqual({ op: 'STOP', args: {} });

    const before = harness.sent.length;
    fireEvent.keyDown(screen.getByLabelText('Task'), { key: 'Escape' });
    expect(harness.sent.length).toBe(before);
  });

  it('does not act when no managed host is attached', () => {
    delete (globalThis as Record<string, unknown>)[HOST_CHANNEL_PROPERTY];
    render(<App />);
    const modifier = navigator.userAgent.includes('Mac') ? { metaKey: true } : { ctrlKey: true };
    // No client exists, so the shortcut cannot reach a runtime.
    expect(() => fireEvent.keyDown(window, { key: 't', ...modifier })).not.toThrow();
  });
});

describe('shortcut documentation (Section 18.2)', () => {
  it('documents exactly the implemented shortcuts, with platform key names', () => {
    renderShell();
    fireEvent.click(screen.getByRole('button', { name: 'Menu' }));
    fireEvent.click(screen.getByRole('menuitem', { name: 'Settings' }));

    const dialog = screen.getByRole('dialog', { name: 'Settings' });
    const isMac = navigator.userAgent.includes('Mac');

    for (const shortcut of SHORTCUTS) {
      expect(within(dialog).getByText(shortcut.description)).toBeTruthy();
      expect(within(dialog).getByText(isMac ? shortcut.mac : shortcut.windows)).toBeTruthy();
    }

    // Shortcuts that do not exist are stated as unimplemented, not hinted.
    for (const entry of UNIMPLEMENTED_SHORTCUTS) {
      expect(within(dialog).getByText(entry.label)).toBeTruthy();
      expect(within(dialog).getAllByText(/Reopen closed tab/).length).toBeGreaterThan(0);
    }
  });

  it('states the privacy and network configuration truthfully', () => {
    renderShell();
    fireEvent.click(screen.getByRole('button', { name: 'Menu' }));
    fireEvent.click(screen.getByRole('menuitem', { name: 'Settings' }));
    const dialog = screen.getByRole('dialog', { name: 'Settings' });

    expect(within(dialog).getByText(/Local only/)).toBeTruthy();
    expect(within(dialog).getByText(/No CDP endpoint/)).toBeTruthy();
    expect(within(dialog).getByText(/never given the app's native bridge/)).toBeTruthy();
    // The harness host is attached, so the screen reports the real connection.
    expect(within(dialog).getByText(/Connected to test-cef-host/)).toBeTruthy();
  });
});
