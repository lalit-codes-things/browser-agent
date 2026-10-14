/**
 * Canonical user-facing copy contract for the Agent panel.
 *
 * These strings are transcribed from the locked specification and must not be
 * re-invented per component:
 *
 * - Task lifecycle labels: Section 2.9.1, [BA-STATE-001]
 * - Wait-reason labels and controls: Sections 2.9.1, 2.9.2, [BA-STATE-002]
 * - Capability readiness labels: [BA-READY-001]
 * - Verification outcome labels: Section 15.5, [BA-VERIFY-001]
 *
 * React renders authoritative runtime state and never synthesizes a state
 * change (Section 12.2), so every value below is a pure presentation mapping
 * from a state already produced by the Rust orchestrator.
 */
import type { AgentReadinessStatus, TaskState, VerificationOutcome, WaitReason } from './types';

/** Every task lifecycle state, in specification order. */
export const TASK_STATES: readonly TaskState[] = [
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

/** States from which no further transition is allowed for that task. */
export const TERMINAL_TASK_STATES: readonly TaskState[] = [
  'BLOCKED',
  'FAILED',
  'CANCELLED',
  'COMPLETED',
];

export function isTerminalTaskState(state: TaskState): boolean {
  return TERMINAL_TASK_STATES.includes(state);
}

/** Primary lifecycle label per state, excluding the reason-dependent wait label. */
const STATE_COPY: Record<Exclude<TaskState, 'WAITING_FOR_USER'>, string> = {
  CREATED: 'Task received',
  NORMALIZING: 'Preparing task',
  OBSERVING: 'Inspecting page',
  PROPOSING: 'Preparing next step',
  POLICY_CHECK: 'Checking permission',
  EXECUTING: 'Performing action',
  VERIFYING: 'Checking result',
  USER_TAKEOVER: "You're in control",
  PAUSED: 'Paused',
  BLOCKED: 'Blocked by policy',
  FAILED: 'Task failed',
  CANCELLED: 'Task cancelled',
  COMPLETED: 'Task ended',
};

/** Every wait reason the schema permits. */
export const WAIT_REASONS: readonly WaitReason[] = [
  'ACTION_APPROVAL',
  'USER_INPUT',
  'AUTHENTICATION',
  'TASK_BUDGET',
];

/** Primary label for a `WAITING_FOR_USER` state, keyed by its typed reason. */
const WAIT_REASON_COPY: Record<WaitReason, string> = {
  ACTION_APPROVAL: 'Waiting for your approval',
  USER_INPUT: 'Needs your input',
  AUTHENTICATION: 'Waiting for you to sign in',
  TASK_BUDGET: 'Task budget reached',
};

/**
 * Shown when a `WAITING_FOR_USER` state carries no valid wait reason. The
 * specification requires failing closed here rather than offering a generic
 * "continue" action (Section 12.2, [BA-STATE-002]).
 */
export const INVALID_WAIT_STATE_COPY = 'Task state is invalid - no wait reason was supplied';

/**
 * Shown when a page action was blocked before dispatch. No verification
 * outcome may be manufactured for an action that never executed (Section 15.5).
 */
export const VERIFICATION_NOT_RUN_COPY = 'Verification not run';

export function isWaitReason(value: unknown): value is WaitReason {
  return typeof value === 'string' && (WAIT_REASONS as readonly string[]).includes(value);
}

/**
 * Primary lifecycle label. `WAITING_FOR_USER` resolves through its typed
 * `waitReason`; a missing or unknown reason fails closed.
 */
export function taskLifecycleLabel(state: TaskState, waitReason: WaitReason | null): string {
  if (state !== 'WAITING_FOR_USER') {
    return STATE_COPY[state];
  }
  return isWaitReason(waitReason) ? WAIT_REASON_COPY[waitReason] : INVALID_WAIT_STATE_COPY;
}

/** True when the wait state is schema-valid and may render controls. */
export function hasValidWaitReason(
  state: TaskState,
  waitReason: WaitReason | null,
): waitReason is WaitReason {
  return state !== 'WAITING_FOR_USER' || isWaitReason(waitReason);
}

/**
 * Capability readiness is a separate typed field from task lifecycle and from
 * verification outcome. Labels are the canonical [BA-READY-001] values.
 */
export const READINESS_STATES: readonly AgentReadinessStatus[] = [
  'NOT_INSTALLED',
  'INSTALLING',
  'INTEGRITY_FAILED',
  'INCOMPATIBLE',
  'LOADING',
  'READY',
  'PARKED',
  'RUNNER_FAILED',
  'RESOURCE_STOPPED',
];

export const READINESS_COPY: Record<AgentReadinessStatus, string> = {
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

/** Verification outcomes, in specification order (Section 15.5). */
export const VERIFICATION_OUTCOMES: readonly VerificationOutcome[] = [
  'VERIFIED_SUCCESS',
  'LIKELY_SUCCESS',
  'UNKNOWN',
  'LIKELY_FAILURE',
  'VERIFIED_FAILURE',
];

export interface VerificationCopy {
  readonly label: string;
  /** Only `VERIFIED_SUCCESS` may use the verified-success treatment. */
  readonly verifiedSuccess: boolean;
}

export const VERIFICATION_COPY: Record<VerificationOutcome, VerificationCopy> = {
  VERIFIED_SUCCESS: { label: 'Verified success', verifiedSuccess: true },
  LIKELY_SUCCESS: { label: 'Likely succeeded — not fully verified', verifiedSuccess: false },
  UNKNOWN: { label: 'Result unknown', verifiedSuccess: false },
  LIKELY_FAILURE: { label: 'Likely did not succeed', verifiedSuccess: false },
  VERIFIED_FAILURE: { label: 'Verified failure', verifiedSuccess: false },
};

/**
 * Outcome copy for an evaluated task. A blocked-before-dispatch action has no
 * outcome and must not display one.
 */
export function verificationCopy(
  outcome: VerificationOutcome | null,
  state: TaskState,
): VerificationCopy {
  if (outcome === null) {
    return {
      label: state === 'BLOCKED' ? VERIFICATION_NOT_RUN_COPY : 'No outcome recorded',
      verifiedSuccess: false,
    };
  }
  return VERIFICATION_COPY[outcome];
}
