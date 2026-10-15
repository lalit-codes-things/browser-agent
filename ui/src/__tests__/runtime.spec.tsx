import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { App } from '../App';
import { SCHEMA_VERSION } from '../bridge/protocol';
import { HOST_CHANNEL_PROPERTY } from '../bridge/host';
import type { HostChannel } from '../bridge/client';

/**
 * Runtime/interface integration (Section 20.2).
 *
 * A stand-in host adapter implements the real channel contract: it receives the
 * JSONL commands the interface sends and pushes the JSONL frames the Rust core
 * emits, byte for byte, from the same schema. Nothing in the application is
 * stubbed: the assertion is that real frames change what the interface renders
 * and that refused frames change nothing.
 */

interface Harness {
  channel: HostChannel;
  /** Lines the interface sent, parsed. */
  sent: { schema_version: number; sequence: number; payload: Record<string, unknown> }[];
  /** Pushes one frame, in the runtime's own envelope format. */
  emit: (payload: Record<string, unknown>, options?: { taskId?: string }) => void;
  /** Pushes a raw line without the harness formatting it. */
  emitRaw: (line: string) => void;
}

function createHarness(): Harness {
  const listeners = new Set<(frame: string) => void>();
  const sent: Harness['sent'] = [];
  let sequence = 0;

  const emitRaw = (line: string) => {
    for (const listener of [...listeners]) listener(line);
  };

  const emit: Harness['emit'] = (payload, options = {}) => {
    sequence += 1;
    const envelope: Record<string, unknown> = {
      schema_version: SCHEMA_VERSION,
      sequence,
      at_unix_ms: 1_700_000_000_000 + sequence,
      monotonic_ms: sequence,
    };
    if (options.taskId !== undefined) envelope.task_id = options.taskId;
    envelope.payload = payload;
    emitRaw(`${JSON.stringify(envelope)}\n`);
  };

  return {
    channel: {
      hostLabel: 'test-cef-host',
      send: (frame: string) => {
        sent.push(JSON.parse(frame) as Harness['sent'][number]);
      },
      subscribe: (listener: (frame: string) => void) => {
        listeners.add(listener);
        return () => {
          listeners.delete(listener);
        };
      },
    },
    sent,
    emit,
    emitRaw,
  };
}

function renderShell(): Harness {
  const harness = createHarness();
  (globalThis as Record<string, unknown>)[HOST_CHANNEL_PROPERTY] = harness.channel;
  render(<App />);
  return harness;
}

function readiness(state: string, label: string, controls: string[] = []): Record<string, unknown> {
  return { kind: 'READINESS', state, label, controls, cause: null, progress: null };
}

function taskStatus(
  state: string,
  waitReason: string | null,
  extras: Record<string, unknown> = {},
): Record<string, unknown> {
  return {
    kind: 'TASK_STATUS',
    status: {
      task_id: 'task-1',
      state,
      label: 'Task received',
      wait_reason: waitReason,
      action_class: null,
      verification: null,
      budget: {
        action_proposals: { limit: 40, used: 2 },
        dispatched_actions: { limit: 25, used: 1 },
        navigations: { limit: 12, used: 1 },
        input_tokens: { limit: 200_000, used: 1_500 },
        output_tokens: { limit: 20_000, used: 300 },
        active_ms: { limit: 600_000, used: 4_000 },
        waiting_ms: 1_000,
        extensions_used: 0,
      },
      elapsed_active_ms: 4_000,
      elapsed_waiting_ms: 1_000,
      last_event_at_unix_ms: 1_700_000_000_000,
      task_tabs: ['tab-1'],
      ...extras,
    },
  };
}

function browserState(): Record<string, unknown> {
  return {
    kind: 'BROWSER_STATE',
    state: {
      window: 'window-1',
      tab: 'tab-1',
      origin: 'https://example.com',
      context: 'ctx-normal',
      private: false,
      address: 'https://example.com/checkout',
      security_label: 'Secure connection',
      title: 'Example checkout',
      loading_state: 'Loaded',
      can_go_back: true,
      can_go_forward: false,
    },
  };
}

function approvalRequest(): Record<string, unknown> {
  return {
    kind: 'APPROVAL_REQUESTED',
    approval: {
      action_id: 'action-1',
      action: 'CLICK',
      action_label: 'Place order',
      origin: 'https://example.com',
      destination: null,
      target_label: 'Place order',
      digest: 'b'.repeat(64),
      epoch: 'epoch-1',
      frame: 'frame-1',
      parameters: [['Amount', '42.00']],
      why: 'This writes an order to the site.',
      rule: 'RULE-IRREVERSIBLE',
      class: 'IRREVERSIBLE',
      expected_effect: 'The order is submitted.',
      undo: null,
      expires_at_unix_ms: 1_700_000_300_000,
    },
  };
}

afterEach(() => {
  delete (globalThis as Record<string, unknown>)[HOST_CHANNEL_PROPERTY];
});

describe('runtime connection', () => {
  it('renders reported browser state and the tab set from host events', () => {
    const harness = renderShell();

    // The host adapter is attached, so the region reports the missing window
    // rather than a missing host.
    expect(screen.getByText('No managed window reported')).toBeTruthy();

    act(() => {
      harness.emit(browserState());
      harness.emit({
        kind: 'TABS',
        tabs: [
          ['tab-1', 'Example checkout'],
          ['tab-2', 'Fares'],
        ],
        active: 'tab-2',
      });
    });

    expect(screen.queryByText('No managed window reported')).toBeNull();
    // The tab title and the page title both come from the host report.
    expect(screen.getAllByText('Example checkout').length).toBeGreaterThan(0);
    expect(screen.getByText('Fares')).toBeTruthy();
    expect(screen.getAllByText(/https:\/\/example\.com\/checkout/).length).toBeGreaterThan(0);
    // Back is available because the host said so; Forward is not.
    expect(screen.getByRole('button', { name: 'Back' }).hasAttribute('disabled')).toBe(false);
    expect(screen.getByRole('button', { name: 'Forward' }).hasAttribute('disabled')).toBe(true);
  });

  it('reports readiness as its own field, separate from any task', () => {
    const harness = renderShell();
    act(() => {
      harness.emit(readiness('LOADING', 'Loading model'));
    });
    expect(screen.getByText('Loading model')).toBeTruthy();
    expect(screen.queryByText('Task received')).toBeNull();
  });

  it('renders the budget, elapsed time and lifecycle the runtime reported', () => {
    const harness = renderShell();
    act(() => {
      harness.emit(taskStatus('EXECUTING', null));
    });
    const panel = screen.getByLabelText('Agent');
    expect(within(panel).getByText('Performing action')).toBeTruthy();
    expect(within(panel).getByText(/steps 2\/40/)).toBeTruthy();
    expect(within(panel).getByText(/actions 1\/25/)).toBeTruthy();
    expect(within(panel).getByText('4s')).toBeTruthy();
    expect(within(panel).getByText('1s')).toBeTruthy();
  });

  it('fails closed when WAITING_FOR_USER arrives without a wait reason', () => {
    const harness = renderShell();
    act(() => {
      harness.emit(taskStatus('WAITING_FOR_USER', null));
    });
    expect(screen.getByText('Task state is invalid - no wait reason was supplied')).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Approve once' })).toBeNull();
  });
});

describe('approval confirmation ([BA-A11Y-002], [BA-POLICY-002])', () => {
  it('shows the exact action, binds the decision and focuses Cancel', () => {
    const harness = renderShell();
    act(() => {
      harness.emit(taskStatus('WAITING_FOR_USER', 'ACTION_APPROVAL'));
      harness.emit(approvalRequest());
    });

    expect(screen.getByText('Place order — your approval is required')).toBeTruthy();
    expect(screen.getByText('https://example.com')).toBeTruthy();
    expect(screen.getByText('This writes an order to the site.')).toBeTruthy();
    expect(screen.getByText('Amount: 42.00')).toBeTruthy();

    const cancel = screen.getByRole('button', { name: 'Cancel' });
    expect(document.activeElement).toBe(cancel);

    fireEvent.click(screen.getByRole('button', { name: 'Approve once' }));
    const approveCommand = harness.sent.at(-1);
    expect(approveCommand?.payload).toEqual({
      domain: 'agent',
      op: 'APPROVE_ACTION',
      args: {
        action_id: 'action-1',
        digest: 'b'.repeat(64),
        origin: 'https://example.com',
        epoch: 'epoch-1',
        frame: 'frame-1',
      },
    });
  });

  it('sends nothing for a decision that does not match the pending request', () => {
    const harness = renderShell();
    act(() => {
      harness.emit(taskStatus('WAITING_FOR_USER', 'ACTION_APPROVAL'));
      harness.emit(approvalRequest());
    });
    act(() => {
      harness.emit({ kind: 'APPROVAL_CLEARED', cause: 'EXPIRED', detail: 'The page changed.' });
    });
    expect(screen.queryByRole('button', { name: 'Approve once' })).toBeNull();
    expect(document.activeElement).toBe(screen.getByLabelText('Agent'));
  });
});

describe('four-part errors ([BA-ERROR-COPY-001])', () => {
  it('renders attempted, observed, known and next action with the code in detail', () => {
    const harness = renderShell();
    act(() => {
      harness.emit({
        kind: 'ERROR',
        error_id: 'BA-ERR-BROWSER-001',
        attempted: 'Browser-Agent tried to open https://example.com.',
        observed: 'The managed browser host is not attached.',
        known: 'Nothing was simulated and no page was opened.',
        next_action: 'Attach the managed browser host and try again.',
        rule: 'RULE-ORIGIN-NOT-ALLOWED',
      });
    });

    expect(screen.getByText('Browser-Agent tried to open https://example.com.')).toBeTruthy();
    expect(screen.getByText('The managed browser host is not attached.')).toBeTruthy();
    expect(screen.getByText('Nothing was simulated and no page was opened.')).toBeTruthy();
    expect(screen.getByText('Attach the managed browser host and try again.')).toBeTruthy();
    // The diagnostic code and the blocking rule stay inside the collapsed
    // detail section, not in the normal view ([BA-ERROR-COPY-001]).
    expect(screen.getByText(/BA-ERR-BROWSER-001/)).toBeTruthy();
    expect(screen.getByText(/RULE-ORIGIN-NOT-ALLOWED/)).toBeTruthy();
  });
});

describe('event stream integrity', () => {
  it('refuses a replayed sequence without changing rendered state', () => {
    const harness = renderShell();
    act(() => {
      harness.emit(readiness('READY', 'Model ready'));
    });
    const stale = JSON.stringify({
      schema_version: SCHEMA_VERSION,
      sequence: 1,
      at_unix_ms: 1_700_000_000_001,
      monotonic_ms: 1,
      payload: readiness('RUNNER_FAILED', 'Model stopped unexpectedly'),
    });

    act(() => {
      harness.emitRaw(`${stale}\n`);
    });

    expect(screen.getByText('Model ready')).toBeTruthy();
    expect(screen.queryByText('Model stopped unexpectedly')).toBeNull();
  });

  it('refuses an unknown event kind without changing rendered state', () => {
    const harness = renderShell();
    act(() => {
      harness.emit(readiness('READY', 'Model ready'));
      harness.emit({ kind: 'TELEPORT', to: 'https://example.com' });
    });
    expect(screen.getByText('Model ready')).toBeTruthy();
  });

  it('reports refused frames in the diagnostic screen', () => {
    const harness = renderShell();
    fireEvent.click(screen.getByRole('button', { name: 'Menu' }));
    fireEvent.click(screen.getByRole('menuitem', { name: 'Audit and diagnostics' }));

    act(() => {
      harness.emit({ kind: 'TELEPORT' });
    });

    const dialog = screen.getByRole('dialog', { name: 'Audit and diagnostics' });
    expect(within(dialog).getByText(/1 frame refused/)).toBeTruthy();
    expect(within(dialog).getByText(/unknown event kind/)).toBeTruthy();
  });
});

describe('secondary screens render real runtime data', () => {
  it('lists task history entries the runtime reported', () => {
    const harness = renderShell();
    act(() => {
      harness.emit({
        kind: 'TASK_HISTORY',
        entries: [
          {
            task_id: 'task-9',
            text: 'open the pricing page',
            state: 'COMPLETED',
            label: 'Task ended',
            verification: 'UNKNOWN',
            verification_label: 'Result unknown',
            created_at_unix_ms: 1_700_000_000_000,
            ended_at_unix_ms: 1_700_000_060_000,
          },
        ],
      });
    });

    fireEvent.click(screen.getByRole('button', { name: 'Menu' }));
    fireEvent.click(screen.getByRole('menuitem', { name: 'Task history' }));

    const dialog = screen.getByRole('dialog', { name: 'Task history' });
    expect(within(dialog).getByText('open the pricing page')).toBeTruthy();
    expect(within(dialog).getByText('Task ended')).toBeTruthy();
    expect(within(dialog).getByText('Result unknown')).toBeTruthy();
    // The screen asks the runtime for its own record.
    expect(harness.sent.at(-1)?.payload.op).toBe('LAST_TASK_RESULT');
  });

  it('offers only the model controls the reported state supports', () => {
    const harness = renderShell();
    act(() => {
      harness.emit(readiness('NOT_INSTALLED', 'Model not installed', ['Install model']));
    });

    fireEvent.click(screen.getByRole('button', { name: 'Menu' }));
    fireEvent.click(screen.getByRole('menuitem', { name: 'Model status' }));

    const dialog = screen.getByRole('dialog', { name: 'Model status' });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Install model' }));
    expect(harness.sent.at(-1)?.payload).toEqual({
      domain: 'model',
      op: 'INSTALL',
      args: { manifest_id: 'qwen3-1.7b-q4_k_m' },
    });
    expect(within(dialog).queryByRole('button', { name: 'Resume model' })).toBeNull();
  });

  it('shows notes and skills from the runtime, and sends only user actions', () => {
    const harness = renderShell();
    act(() => {
      harness.emit({
        kind: 'NOTES',
        notes: [
          {
            note: 'note-1',
            title: 'Fare rules',
            body: 'Peak hours cost more.',
            source_url: 'https://example.com/fares',
            captured_at_unix_ms: 1_700_000_000_000,
          },
        ],
      });
      harness.emit({
        kind: 'SKILLS',
        skills: [
          {
            skill: 'skill-1',
            name: 'Open the fare table',
            version: '1',
            provenance: 'saved from task-1 and reviewed',
            enabled: true,
          },
        ],
      });
    });

    fireEvent.click(screen.getByRole('button', { name: 'Menu' }));
    fireEvent.click(screen.getByRole('menuitem', { name: 'Skills and notes' }));

    const dialog = screen.getByRole('dialog', { name: 'Skills and notes' });
    expect(within(dialog).getByText('Open the fare table — version 1')).toBeTruthy();
    expect(within(dialog).getByText('Fare rules')).toBeTruthy();
    expect(within(dialog).getByText('Peak hours cost more.')).toBeTruthy();

    fireEvent.click(within(dialog).getByRole('button', { name: 'Disable skill' }));
    expect(harness.sent.at(-1)?.payload).toEqual({
      domain: 'data',
      op: 'REVOKE_SKILL',
      args: { skill: 'skill-1' },
    });
  });

  it('lists downloads with their provenance and reported progress', () => {
    const harness = renderShell();
    act(() => {
      harness.emit({
        kind: 'DOWNLOADS',
        downloads: [
          {
            file_name: 'invoice.pdf',
            origin: 'https://example.com',
            provenance: 'Requested by Agent task task-3',
            progress: '100%',
          },
        ],
      });
    });

    fireEvent.click(screen.getByRole('button', { name: 'Menu' }));
    fireEvent.click(screen.getByRole('menuitem', { name: 'Downloads' }));

    const dialog = screen.getByRole('dialog', { name: 'Downloads' });
    expect(within(dialog).getByText('invoice.pdf')).toBeTruthy();
    expect(within(dialog).getByText('Requested by Agent task task-3')).toBeTruthy();
  });
});
