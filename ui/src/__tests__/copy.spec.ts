import {
  INVALID_WAIT_STATE_COPY,
  READINESS_COPY,
  READINESS_STATES,
  TASK_STATES,
  VERIFICATION_COPY,
  VERIFICATION_NOT_RUN_COPY,
  WAIT_REASONS,
  hasValidWaitReason,
  isTerminalTaskState,
  taskLifecycleLabel,
  verificationCopy,
} from '../domain/copy';
import type { TaskState, WaitReason } from '../domain/types';

/**
 * [BA-STATE-001] / [BA-STATE-002] / [BA-READY-001] / [BA-VERIFY-001]: the copy
 * contract is a table, so every state, reason, readiness value and outcome is
 * asserted rather than a sample.
 */

describe('task lifecycle copy ([BA-STATE-001])', () => {
  const expected: Record<TaskState, string | null> = {
    CREATED: 'Task received',
    NORMALIZING: 'Preparing task',
    OBSERVING: 'Inspecting page',
    PROPOSING: 'Preparing next step',
    POLICY_CHECK: 'Checking permission',
    EXECUTING: 'Performing action',
    VERIFYING: 'Checking result',
    WAITING_FOR_USER: null,
    USER_TAKEOVER: "You're in control",
    PAUSED: 'Paused',
    BLOCKED: 'Blocked by policy',
    FAILED: 'Task failed',
    CANCELLED: 'Task cancelled',
    COMPLETED: 'Task ended',
  };

  it('covers every declared task state', () => {
    expect(Object.keys(expected).sort()).toEqual([...TASK_STATES].sort());
  });

  it.each(Object.entries(expected).filter(([, label]) => label !== null))(
    'maps %s to its canonical label',
    (state, label) => {
      expect(taskLifecycleLabel(state as TaskState, null)).toBe(label);
    },
  );

  it('resolves WAITING_FOR_USER through its typed reason', () => {
    const expectedByReason: Record<WaitReason, string> = {
      ACTION_APPROVAL: 'Waiting for your approval',
      USER_INPUT: 'Needs your input',
      AUTHENTICATION: 'Waiting for you to sign in',
      TASK_BUDGET: 'Task budget reached',
    };

    expect(Object.keys(expectedByReason).sort()).toEqual([...WAIT_REASONS].sort());

    for (const [reason, label] of Object.entries(expectedByReason)) {
      expect(taskLifecycleLabel('WAITING_FOR_USER', reason as WaitReason)).toBe(label);
    }
  });

  it('fails closed when WAITING_FOR_USER has no valid reason ([BA-STATE-002])', () => {
    expect(taskLifecycleLabel('WAITING_FOR_USER', null)).toBe(INVALID_WAIT_STATE_COPY);
    expect(hasValidWaitReason('WAITING_FOR_USER', null)).toBe(false);
    expect(hasValidWaitReason('WAITING_FOR_USER', 'ACTION_APPROVAL')).toBe(true);
    expect(hasValidWaitReason('EXECUTING', null)).toBe(true);
  });

  it('treats only the four documented states as terminal', () => {
    const terminal = TASK_STATES.filter((state) => isTerminalTaskState(state));
    expect(terminal).toEqual(['BLOCKED', 'FAILED', 'CANCELLED', 'COMPLETED']);
  });
});

describe('capability readiness copy ([BA-READY-001])', () => {
  const expected: Record<string, string> = {
    NOT_INSTALLED: 'Model not installed',
    INSTALLING: 'Installing model',
    INTEGRITY_FAILED: 'Model failed its integrity check',
    INCOMPATIBLE: 'Model not compatible with this computer',
    LOADING: 'Loading model',
    READY: 'Model ready',
    PARKED: 'Model paused to free memory',
    RUNNER_FAILED: 'Model stopped unexpectedly',
    RESOURCE_STOPPED: 'Model stopped to protect system performance',
  };

  it('covers every declared readiness state', () => {
    expect(Object.keys(READINESS_COPY).sort()).toEqual([...READINESS_STATES].sort());
    expect(Object.keys(expected).sort()).toEqual([...READINESS_STATES].sort());
  });

  it.each(Object.entries(expected))('labels %s exactly', (state, label) => {
    expect(READINESS_COPY[state as keyof typeof READINESS_COPY]).toBe(label);
  });

  it('keeps readiness copy distinct from task lifecycle copy', () => {
    const lifecycle = TASK_STATES.map((state) => taskLifecycleLabel(state, 'ACTION_APPROVAL'));
    for (const label of Object.values(READINESS_COPY)) {
      expect(lifecycle).not.toContain(label);
    }
  });
});

describe('verification outcome copy (Section 15.5)', () => {
  it('labels each outcome and reserves the success treatment', () => {
    expect(verificationCopy('VERIFIED_SUCCESS', 'COMPLETED')).toEqual({
      label: 'Verified success',
      verifiedSuccess: true,
    });
    expect(verificationCopy('LIKELY_SUCCESS', 'COMPLETED')).toEqual({
      label: 'Likely succeeded — not fully verified',
      verifiedSuccess: false,
    });
    expect(verificationCopy('UNKNOWN', 'COMPLETED')).toEqual({
      label: 'Result unknown',
      verifiedSuccess: false,
    });
    expect(verificationCopy('LIKELY_FAILURE', 'FAILED')).toEqual({
      label: 'Likely did not succeed',
      verifiedSuccess: false,
    });
    expect(verificationCopy('VERIFIED_FAILURE', 'COMPLETED')).toEqual({
      label: 'Verified failure',
      verifiedSuccess: false,
    });

    expect(VERIFICATION_COPY.VERIFIED_SUCCESS.verifiedSuccess).toBe(true);
    for (const outcome of ['LIKELY_SUCCESS', 'UNKNOWN', 'LIKELY_FAILURE', 'VERIFIED_FAILURE']) {
      expect(VERIFICATION_COPY[outcome as keyof typeof VERIFICATION_COPY].verifiedSuccess).toBe(
        false,
      );
    }
  });

  it('does not manufacture an outcome for an action blocked before dispatch', () => {
    expect(verificationCopy(null, 'BLOCKED').label).toBe(VERIFICATION_NOT_RUN_COPY);
    expect(verificationCopy(null, 'BLOCKED').verifiedSuccess).toBe(false);
  });
});

describe('task lifecycle and verification are independent fields', () => {
  it('renders FAILED with UNKNOWN as two separate statements', () => {
    expect(taskLifecycleLabel('FAILED', null)).toBe('Task failed');
    expect(verificationCopy('UNKNOWN', 'FAILED').label).toBe('Result unknown');
  });

  it('renders COMPLETED with VERIFIED_FAILURE without calling it a failure of the run', () => {
    expect(taskLifecycleLabel('COMPLETED', null)).toBe('Task ended');
    expect(verificationCopy('VERIFIED_FAILURE', 'COMPLETED').label).toBe('Verified failure');
  });
});
