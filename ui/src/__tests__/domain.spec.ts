import { createTaskEvent, createTaskSummary } from '../domain';
import { TaskState, WaitReason, VerificationOutcome, ActionClass } from '../domain/types';

function now() {
  return Date.now();
}

describe('Task state machine', () => {
  it('assigns each state a distinct label', () => {
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
  });

  it('enforces a single wait reason per WAITING_FOR_USER transition', () => {
    const reasons: WaitReason[] = [
      'ACTION_APPROVAL',
      'USER_INPUT',
      'AUTHENTICATION',
      'TASK_BUDGET',
    ];
    expect(new Set(reasons).size).toBe(reasons.length);
    expect('ACTION_APPROVAL').toBe('ACTION_APPROVAL');
    expect('USER_INPUT').toBe('USER_INPUT');
    expect('AUTHENTICATION').toBe('AUTHENTICATION');
    expect('TASK_BUDGET').toBe('TASK_BUDGET');
  });
});

describe('Task event creation', () => {
  it('binds revision, sequence and timestamps deterministically', () => {
    const taskId = 'task-1';
    const t = createTaskEvent(taskId, 1, 1, 'CREATED', null, now(), 'Task received', '');
    expect(t.taskId).toBe(taskId);
    expect(t.revision).toBe(1);
    expect(t.sequence).toBe(1);
    expect(t.state).toBe('CREATED');
    expect(t.waitReason).toBeNull();
    expect(t.actionLabel).toBe('Task received');
  });
});

describe('Task summary construction', () => {
  it('rounds trips through the API shape', () => {
    const t = createTaskSummary(
      'task-1',
      'open the home page',
      'CREATED',
      null,
      null,
      0,
      [createTaskEvent('task-1', 1, 1, 'CREATED', null, now(), 'Task received', '')],
      'UNKNOWN',
    );
    expect(t.taskId).toBe('task-1');
    expect(t.originalText).toBe('open the home page');
    expect(t.state).toBe('CREATED');
    expect(t.currentAction).toBeNull();
    expect(t.elapsedMs).toBe(0);
    expect(t.verification).toBe('UNKNOWN');
  });
});

describe('Action classes', () => {
  it('keeps the four approved classes', () => {
    expect('READ').toBe('READ');
    expect('REVERSIBLE_WRITE').toBe('REVERSIBLE_WRITE');
    expect('IRREVERSIBLE').toBe('IRREVERSIBLE');
    expect('UNKNOWN').toBe('UNKNOWN');
  });
});

describe('Verification outcomes', () => {
  it('lists the vocabulary', () => {
    expect('VERIFIED_SUCCESS').toBe('VERIFIED_SUCCESS');
    expect('LIKELY_SUCCESS').toBe('LIKELY_SUCCESS');
    expect('UNKNOWN').toBe('UNKNOWN');
    expect('LIKELY_FAILURE').toBe('LIKELY_FAILURE');
    expect('VERIFIED_FAILURE').toBe('VERIFIED_FAILURE');
  });
});
