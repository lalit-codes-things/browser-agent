import type { RuntimeSnapshot } from '../bridge/client';
import type { ReadinessView, TaskStatusView } from '../bridge/protocol';
import { READINESS_COPY, TASK_STATES, isTerminalTaskState } from './copy';
import type { AgentPanelState, AgentReadiness, AgentReadinessStatus, TaskState } from './types';

/**
 * Projection from validated runtime events to what the Agent panel renders.
 *
 * Every value here is copied from a frame the runtime sent. React never
 * advances a lifecycle, never estimates a budget, never invents a progress
 * percentage and never reports verification success on its own (Section 12.2).
 */

/** Narrows an arbitrary runtime state code to a state the UI may render. */
export function asTaskState(value: string): TaskState | null {
  return (TASK_STATES as readonly string[]).includes(value) ? (value as TaskState) : null;
}

/** Narrows an arbitrary readiness code to one the UI may render. */
export function asReadinessStatus(value: string): AgentReadinessStatus | null {
  return Object.prototype.hasOwnProperty.call(READINESS_COPY, value)
    ? (value as AgentReadinessStatus)
    : null;
}

/**
 * Shown when the runtime reports a state code this build does not know. The UI
 * fails closed: it displays the honest statement and offers no control, rather
 * than guessing an operation.
 */
export const UNKNOWN_READINESS_COPY = 'Model state not recognised by this build';

/**
 * No model can be loaded until the managed browser host and the local runner
 * exist. Absence is reported as the canonical `NOT_INSTALLED` readiness state
 * with no fabricated detail ([BA-READY-001]).
 */
export function createAgentReadiness(): AgentReadiness {
  return {
    status: 'NOT_INSTALLED',
    label: READINESS_COPY.NOT_INSTALLED,
    message: '',
  };
}

export function createAgentPanelState(): AgentPanelState {
  return {
    readiness: createAgentReadiness(),
    status: null,
    activity: [],
    ended: null,
    approval: null,
    handoff: null,
    error: null,
    composerLocked: false,
  };
}

/**
 * Readiness as reported by the runtime. An unrecognised code renders
 * `UNKNOWN_READINESS_COPY` with no controls.
 */
export function readinessFromView(view: ReadinessView | null): AgentReadiness {
  if (view === null) return createAgentReadiness();
  const status = asReadinessStatus(view.state);
  if (status === null) {
    return { status: 'RUNNER_FAILED', label: UNKNOWN_READINESS_COPY, message: view.cause ?? '' };
  }
  return {
    status,
    label: view.label,
    message: view.cause ?? view.progress ?? '',
  };
}

function isLocked(status: TaskStatusView | null): boolean {
  if (status === null) return false;
  const state = asTaskState(status.state);
  return state !== null && !isTerminalTaskState(state);
}

/** Builds the panel projection from the latest validated runtime snapshot. */
export function panelStateFromSnapshot(snapshot: RuntimeSnapshot): AgentPanelState {
  const status = snapshot.task;
  return {
    readiness: readinessFromView(snapshot.readiness),
    status,
    activity: [...snapshot.activity],
    ended: snapshot.ended,
    approval: snapshot.approval,
    handoff: snapshot.handoff,
    error: snapshot.error,
    composerLocked: isLocked(status),
  };
}
