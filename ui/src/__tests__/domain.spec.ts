import {
  panelStateFromSnapshot,
  readinessFromView,
  asTaskState,
  asReadinessStatus,
  UNKNOWN_READINESS_COPY,
} from '../domain/state';
import type { RuntimeSnapshot } from '../bridge/client';
import type { TaskState, WaitReason } from '../domain/types';

/**
 * Projection from validated runtime events to renderable state.
 *
 * These tests use the exact wire shapes `crates/ba-ipc` emits, so a change to
 * the Rust field names or value vocabulary fails here rather than rendering an
 * empty field in the interface.
 */

function snapshot(overrides: Partial<RuntimeSnapshot>): RuntimeSnapshot {
  return {
    attached: true,
    browser: null,
    tabs: [],
    activeTab: null,
    readiness: null,
    task: null,
    activity: [],
    ended: null,
    approval: null,
    handoff: null,
    history: null,
    notes: null,
    skills: null,
    downloads: null,
    audit: null,
    error: null,
    lastRefusal: null,
    refusals: 0,
    ...overrides,
  };
}

function status(state: string, waitReason: string | null) {
  return {
    task_id: 'task-1',
    state,
    label: 'Task received',
    wait_reason: waitReason,
    action_class: null,
    verification: null,
    budget: {
      action_proposals: { limit: 40, used: 1 },
      dispatched_actions: { limit: 25, used: 0 },
      navigations: { limit: 12, used: 0 },
      input_tokens: { limit: 200000, used: 0 },
      output_tokens: { limit: 20000, used: 0 },
      active_ms: { limit: 600000, used: 0 },
      waiting_ms: 0,
      extensions_used: 0,
    },
    elapsed_active_ms: 0,
    elapsed_waiting_ms: 0,
    last_event_at_unix_ms: 1_700_000_000_000,
    task_tabs: ['tab-1'],
  };
}

describe('task state vocabulary', () => {
  it('accepts only declared states and reasons', () => {
    const states: TaskState[] = [
      'CREATED',
      'NORMALIZING',
      'OBSERVING',
      'PROPOSING',
      'POLICY_CHECK',
      'EXECUTING',
      'VERIFYING',
      'WAITING_FOR_USER',
      'USER_TAKEOVER',
      'PAUSED',
      'BLOCKED',
      'FAILED',
      'CANCELLED',
      'COMPLETED',
    ];
    expect(new Set(states).size).toBe(states.length);
    for (const state of states) {
      expect(asTaskState(state)).toBe(state);
    }
    expect(asTaskState('TELEPORTING')).toBeNull();

    const reasons: WaitReason[] = [
      'ACTION_APPROVAL',
      'USER_INPUT',
      'AUTHENTICATION',
      'TASK_BUDGET',
    ];
    expect(new Set(reasons).size).toBe(reasons.length);
  });
});

describe('readiness projection ([BA-READY-001])', () => {
  it('uses the canonical runtime label and its measured cause', () => {
    const readiness = readinessFromView({
      state: 'INSTALLING',
      label: 'Installing model',
      controls: ['Cancel'],
      cause: 'Downloading artifact',
      progress: '42% (440 of 1048 MB)',
    });
    expect(readiness.status).toBe('INSTALLING');
    expect(readiness.label).toBe('Installing model');
    expect(readiness.message).toBe('Downloading artifact');
  });

  it('fails closed for a state code this build does not know', () => {
    const readiness = readinessFromView({
      state: 'QUANTUM',
      label: 'Something else',
      controls: [],
      cause: null,
      progress: null,
    });
    expect(readiness.label).toBe(UNKNOWN_READINESS_COPY);
    expect(asReadinessStatus('QUANTUM')).toBeNull();
  });

  it('reports no installed model when the runtime has said nothing', () => {
    const readiness = readinessFromView(null);
    expect(readiness.status).toBe('NOT_INSTALLED');
    expect(readiness.label).toBe('Model not installed');
    expect(readiness.message).toBe('');
  });
});

describe('panel projection', () => {
  it('keeps readiness, lifecycle, budget and verification independent', () => {
    const panel = panelStateFromSnapshot(
      snapshot({
        readiness: {
          state: 'READY',
          label: 'Model ready',
          controls: [],
          cause: null,
          progress: null,
        },
        task: status('WAITING_FOR_USER', 'ACTION_APPROVAL'),
      }),
    );
    expect(panel.readiness.label).toBe('Model ready');
    expect(panel.status?.state).toBe('WAITING_FOR_USER');
    expect(panel.status?.wait_reason).toBe('ACTION_APPROVAL');
    expect(panel.status?.budget.dispatched_actions.limit).toBe(25);
    expect(panel.ended).toBeNull();
  });

  it('locks the composer only for a non-terminal task', () => {
    expect(
      panelStateFromSnapshot(snapshot({ task: status('EXECUTING', null) })).composerLocked,
    ).toBe(true);
    expect(
      panelStateFromSnapshot(snapshot({ task: status('COMPLETED', null) })).composerLocked,
    ).toBe(false);
    expect(panelStateFromSnapshot(snapshot({})).composerLocked).toBe(false);
  });

  it('keeps the terminal report separate from an active status', () => {
    const panel = panelStateFromSnapshot(
      snapshot({
        ended: {
          task_id: 'task-1',
          state: 'COMPLETED',
          state_label: 'Task ended',
          verification: 'UNKNOWN',
          verification_label: 'Result unknown',
          detail: 'The page did not expose a confirmation.',
        },
      }),
    );
    expect(panel.status).toBeNull();
    expect(panel.ended?.state_label).toBe('Task ended');
    expect(panel.ended?.verification_label).toBe('Result unknown');
  });
});
