import {
  EVENT_KINDS,
  MAX_IPC_FRAME_BYTES,
  SCHEMA_VERSION,
  decodeEventLine,
  encodeCommandLine,
} from '../bridge/protocol';
import { commands } from '../bridge/client';

/**
 * Inbound frame validation (Section 20.2).
 *
 * Every case here is a frame the runtime must never be able to smuggle past the
 * boundary: an oversized frame, a malformed payload, an unknown kind, a wrong
 * schema version, or extra fields beside a payload. The decoder rebuilds each
 * payload from validated values, so an injected property has no path into
 * application state.
 */

const READINESS_FRAME = JSON.stringify({
  schema_version: SCHEMA_VERSION,
  sequence: 4,
  at_unix_ms: 1_700_000_000_000,
  monotonic_ms: 900,
  payload: {
    kind: 'READINESS',
    state: 'NOT_INSTALLED',
    label: 'Model not installed',
    controls: ['Install model'],
    cause: null,
    progress: null,
  },
});

function frameWith(payload: unknown, overrides: Record<string, unknown> = {}): string {
  return JSON.stringify({
    schema_version: SCHEMA_VERSION,
    sequence: 1,
    at_unix_ms: 1_700_000_000_000,
    monotonic_ms: 10,
    payload,
    ...overrides,
  });
}

describe('event decoding', () => {
  it('accepts a well-formed readiness event', () => {
    const result = decodeEventLine(`${READINESS_FRAME}\n`);
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.event.sequence).toBe(4);
    expect(result.event.taskId).toBeNull();
    expect(result.event.payload.kind).toBe('READINESS');
    if (result.event.payload.kind !== 'READINESS') return;
    expect(result.event.payload.readiness.state).toBe('NOT_INSTALLED');
    expect(result.event.payload.readiness.controls).toEqual(['Install model']);
  });

  it('declares every kind the runtime emits', () => {
    expect([...EVENT_KINDS].sort()).toEqual(
      [
        'TASK',
        'TASK_STATUS',
        'TASK_ENDED',
        'BROWSER_STATE',
        'TABS',
        'READINESS',
        'APPROVAL_REQUESTED',
        'APPROVAL_CLEARED',
        'HANDOFF_REQUIRED',
        'TASK_HISTORY',
        'NOTES',
        'SKILLS',
        'DOWNLOADS',
        'AUDIT_STATUS',
        'ERROR',
        'COMMAND_REJECTED',
      ].sort(),
    );
  });

  it('refuses empty, oversized, malformed and wrong-version frames', () => {
    expect(decodeEventLine('   ')).toMatchObject({ ok: false, failure: 'EMPTY_FRAME' });
    expect(decodeEventLine('{not json')).toMatchObject({ ok: false, failure: 'INVALID_JSON' });

    const oversized = JSON.stringify({
      schema_version: SCHEMA_VERSION,
      sequence: 1,
      at_unix_ms: 0,
      monotonic_ms: 0,
      payload: { kind: 'COMMAND_REJECTED', command: 'x', reason: 'y'.repeat(MAX_IPC_FRAME_BYTES) },
    });
    expect(decodeEventLine(oversized)).toMatchObject({ ok: false, failure: 'OVERSIZED_FRAME' });

    const wrongVersion = frameWith(
      { kind: 'APPROVAL_CLEARED', cause: 'EXPIRED', detail: 'The request expired.' },
      { schema_version: 99 },
    );
    expect(decodeEventLine(wrongVersion)).toMatchObject({
      ok: false,
      failure: 'SCHEMA_MISMATCH',
    });
  });

  it('refuses an unknown kind and a payload that does not match its schema', () => {
    expect(decodeEventLine(frameWith({ kind: 'TELEPORT' }))).toMatchObject({
      ok: false,
      failure: 'UNKNOWN_KIND',
    });
    expect(
      decodeEventLine(frameWith({ kind: 'READINESS', state: 'READY', controls: [] })),
    ).toMatchObject({ ok: false, failure: 'MALFORMED_PAYLOAD' });
    // A wait reason of the wrong type must not be coerced.
    expect(
      decodeEventLine(
        frameWith({
          kind: 'TASK_STATUS',
          status: {
            task_id: 'task-1',
            state: 'EXECUTING',
            label: 'Performing action',
            wait_reason: 7,
            action_class: null,
            verification: null,
            budget: {
              action_proposals: { limit: 1, used: 0 },
              dispatched_actions: { limit: 1, used: 0 },
              navigations: { limit: 1, used: 0 },
              input_tokens: { limit: 1, used: 0 },
              output_tokens: { limit: 1, used: 0 },
              active_ms: { limit: 1, used: 0 },
              waiting_ms: 0,
              extensions_used: 0,
            },
            elapsed_active_ms: 0,
            elapsed_waiting_ms: 0,
            last_event_at_unix_ms: 1,
            task_tabs: [],
          },
        }),
      ),
    ).toMatchObject({ ok: false, failure: 'MALFORMED_PAYLOAD' });
  });

  it('drops fields the schema does not declare instead of carrying them into state', () => {
    const result = decodeEventLine(
      frameWith({
        kind: 'APPROVAL_CLEARED',
        cause: 'EXPIRED',
        detail: 'The request expired.',
        injected: 'grant-approval',
      }),
    );
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(Object.keys(result.event.payload)).toEqual(['kind', 'cause', 'detail']);
  });

  it('requires the approval binding the decision is checked against', () => {
    const approval = {
      action_id: 'action-1',
      action: 'CLICK',
      action_label: 'Place order',
      origin: 'https://example.com',
      destination: null,
      target_label: 'Place order',
      digest: 'a'.repeat(64),
      epoch: 'epoch-1',
      frame: 'frame-1',
      parameters: [['Amount', '42.00']],
      why: 'This writes an order.',
      rule: 'RULE-IRREVERSIBLE-WRITE',
      class: 'IRREVERSIBLE',
      expected_effect: 'An order is placed.',
      undo: null,
      expires_at_unix_ms: 1_700_000_300_000,
    };

    const accepted = decodeEventLine(frameWith({ kind: 'APPROVAL_REQUESTED', approval }));
    expect(accepted.ok).toBe(true);
    if (!accepted.ok || accepted.event.payload.kind !== 'APPROVAL_REQUESTED') return;
    expect(accepted.event.payload.approval.digest).toBe('a'.repeat(64));
    expect(accepted.event.payload.approval.epoch).toBe('epoch-1');
    expect(accepted.event.payload.approval.frame).toBe('frame-1');

    const withoutBinding: Record<string, unknown> = { ...approval };
    delete withoutBinding.digest;
    expect(
      decodeEventLine(frameWith({ kind: 'APPROVAL_REQUESTED', approval: withoutBinding })),
    ).toMatchObject({ ok: false, failure: 'MALFORMED_PAYLOAD' });
  });

  it('reads a frame that reports no managed browser state', () => {
    const result = decodeEventLine(frameWith({ kind: 'BROWSER_STATE', state: null }));
    expect(result.ok).toBe(true);
    if (!result.ok || result.event.payload.kind !== 'BROWSER_STATE') return;
    expect(result.event.payload.state).toBeNull();
  });
});

describe('command encoding', () => {
  it('writes one versioned JSONL line with an argument object for every operation', () => {
    const line = encodeCommandLine(commands.modelStatus(), 3, 1_700_000_000_000);
    expect(line.endsWith('\n')).toBe(true);
    expect(line.trimEnd().split('\n')).toHaveLength(1);

    const parsed = JSON.parse(line) as {
      schema_version: number;
      sequence: number;
      at_unix_ms: number;
      payload: { domain: string; op: string; args: Record<string, unknown> };
    };
    expect(parsed.schema_version).toBe(SCHEMA_VERSION);
    expect(parsed.sequence).toBe(3);
    expect(parsed.at_unix_ms).toBe(1_700_000_000_000);
    expect(parsed.payload).toEqual({ domain: 'model', op: 'STATUS', args: {} });
  });

  it('sends only typed arguments the Rust core declares', () => {
    const submit = JSON.parse(
      encodeCommandLine(commands.submitTask('buy milk', 'window-1', 'tab-1'), 1, 0),
    ) as {
      payload: { domain: string; op: string; args: Record<string, unknown> };
    };
    expect(submit.payload.domain).toBe('agent');
    expect(submit.payload.op).toBe('SUBMIT_TASK');
    // The origin is deliberately absent: the runtime takes it from its own state.
    expect(Object.keys(submit.payload.args).sort()).toEqual(['tab', 'text', 'window']);

    const approve = JSON.parse(
      encodeCommandLine(commands.approveAction('action-1', 'a'.repeat(64), 'o', 'e', 'f'), 2, 0),
    ) as { payload: { args: Record<string, unknown> } };
    expect(Object.keys(approve.payload.args).sort()).toEqual([
      'action_id',
      'digest',
      'epoch',
      'frame',
      'origin',
    ]);
  });
});
