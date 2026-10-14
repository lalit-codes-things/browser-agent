import { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react';
import type { AgentPanelState, ApprovalDetails, TaskSummary } from './types';
import { READINESS_COPY, isTerminalTaskState, isWaitReason } from './copy';
import { createAgentPanelState } from './state';
import { useBrowser } from './useBrowser';

/**
 * Agent panel state is rendered from the Rust orchestrator. Nothing in this
 * module may synthesize a task state, advance a lifecycle with a timer, or
 * report verification success: React renders authoritative runtime state and
 * cannot invent transitions (Section 12.2). When no native host is attached the
 * panel reports that honestly instead of demonstrating a simulated run.
 */

/** Composer ceiling in Unicode code points ([BA-COMPOSER-001]). */
export const MAX_TASK_CODE_POINTS = 8000;

/** Real count of Unicode code points, not UTF-16 units. */
export function countCodePoints(text: string): number {
  return Array.from(text).length;
}

/**
 * Bridge event kinds the UI expects from the native host. The transports and
 * envelope are owned by the CEF/native contract (ADR-010); the UI only renders
 * what arrives and never fabricates a payload.
 */
export const AGENT_EVENT_KINDS = {
  readiness: 'agent.readiness',
  task: 'agent.task',
} as const;

export const RUNTIME_NOT_CONNECTED_COPY = 'Agent runtime not connected';

export const RUNTIME_NOT_CONNECTED_DETAIL =
  'The native browser host is not attached in this build, so no model can be installed or loaded and no task can start.';

export const TASK_NOT_STARTED_COPY =
  'No task was started: the agent runtime is not connected, so the request was not sent anywhere.';

export const COMPOSER_OVER_LIMIT_COPY =
  'This task exceeds the 8,000 code point limit. Shorten it before submitting; the text is not truncated.';

export interface AgentContextValue {
  state: AgentPanelState;
  /** True only when a real native host has attached its bridge. */
  runtimeConnected: boolean;
  /** Honest, non-simulated status message for the panel. */
  notice: string | null;
  submitTask: (text: string) => Promise<string>;
  pause: () => Promise<void>;
  cancel: () => Promise<void>;
  takeOver: () => Promise<void>;
  resume: () => Promise<void>;
  respondToApproval: (actionId: string, approved: boolean) => Promise<void>;
}

const AgentContext = createContext<AgentContextValue | null>(null);

function isTaskSummary(value: unknown): value is TaskSummary {
  if (typeof value !== 'object' || value === null) return false;
  const candidate = value as Partial<TaskSummary>;
  return typeof candidate.taskId === 'string' && typeof candidate.state === 'string';
}

function isApprovalDetails(value: unknown): value is ApprovalDetails {
  if (typeof value !== 'object' || value === null) return false;
  const candidate = value as Partial<ApprovalDetails>;
  return typeof candidate.actionId === 'string' && typeof candidate.action === 'object';
}

export function AgentProvider({ children }: { children: React.ReactNode }) {
  const browser = useBrowser();
  const runtimeConnected = browser.bridgeAttached;

  const [panel, setPanel] = useState<AgentPanelState>(() => createAgentPanelState());
  const [notice, setNotice] = useState<string | null>(null);

  // Render authoritative state emitted by the runtime.
  useEffect(() => {
    if (!runtimeConnected) return undefined;
    return browser.onEvent((event) => {
      if (event.kind === AGENT_EVENT_KINDS.readiness) {
        const payload = event.payload as { status?: unknown; message?: unknown } | null;
        const status = payload?.status;
        if (typeof status === 'string' && status in READINESS_COPY) {
          setPanel((prev) => ({
            ...prev,
            readiness: {
              status: status as AgentPanelState['readiness']['status'],
              message: typeof payload?.message === 'string' ? payload.message : '',
            },
          }));
        }
        return;
      }
      if (event.kind === AGENT_EVENT_KINDS.task) {
        const payload = event.payload;
        if (payload === null) {
          setPanel((prev) => ({ ...prev, task: null, approval: null }));
          return;
        }
        if (!isTaskSummary(payload)) return;
        // A `WAITING_FOR_USER` state without a typed reason is schema-invalid;
        // drop the reason so copy and controls fail closed instead of showing a
        // generic continue action ([BA-STATE-002]).
        const waitReason = isWaitReason(payload.waitReason) ? payload.waitReason : null;
        setPanel((prev) => ({
          ...prev,
          task: { ...payload, waitReason },
          approval: null,
        }));
        return;
      }
      if (event.kind === 'agent.approval') {
        setPanel((prev) => ({
          ...prev,
          approval: isApprovalDetails(event.payload) ? event.payload : null,
        }));
      }
    });
  }, [browser, runtimeConnected]);

  const submitTask = useCallback(
    async (text: string): Promise<string> => {
      const trimmed = text.trim();
      if (trimmed.length === 0) return '';

      if (countCodePoints(trimmed) > MAX_TASK_CODE_POINTS) {
        setNotice(COMPOSER_OVER_LIMIT_COPY);
        return '';
      }

      // One active autonomous task process-wide ([BA-WIN-002]). A task already
      // in flight must be resolved before another is submitted.
      if (panel.task !== null && !isTerminalTaskState(panel.task.state)) {
        return '';
      }

      if (!runtimeConnected) {
        // Never fabricate a task or a lifecycle for a runtime that is absent.
        setNotice(TASK_NOT_STARTED_COPY);
        return '';
      }

      setNotice(null);
      return browser.agentSubmitTask(trimmed);
    },
    [browser, panel.task, runtimeConnected],
  );

  const forward = useCallback(
    (method: () => Promise<void>) => async (): Promise<void> => {
      if (!runtimeConnected) return;
      await method();
    },
    [runtimeConnected],
  );

  const pause = useMemo(() => forward(() => browser.agentPause()), [browser, forward]);
  const cancel = useMemo(() => forward(() => browser.agentCancel()), [browser, forward]);
  const takeOver = useMemo(() => forward(() => browser.agentTakeOver()), [browser, forward]);
  const resume = useMemo(() => forward(() => browser.agentResume()), [browser, forward]);

  const respondToApproval = useCallback(
    async (actionId: string, approved: boolean): Promise<void> => {
      if (!runtimeConnected || panel.approval === null) return;
      // The approval is single-use and expires; the host owns consumption.
      await browser.agentRespondToApproval(actionId, approved);
    },
    [browser, panel.approval, runtimeConnected],
  );

  const contextValue = useMemo<AgentContextValue>(() => {
    const task = panel.task;
    const composerLocked = task !== null && !isTerminalTaskState(task.state);
    return {
      state: { ...panel, composerLocked },
      runtimeConnected,
      notice,
      submitTask,
      pause,
      cancel,
      takeOver,
      resume,
      respondToApproval,
    };
  }, [
    panel,
    runtimeConnected,
    notice,
    submitTask,
    pause,
    cancel,
    takeOver,
    resume,
    respondToApproval,
  ]);

  return <AgentContext.Provider value={contextValue}>{children}</AgentContext.Provider>;
}

export function useAgent(): AgentContextValue {
  const ctx = useContext(AgentContext);
  if (ctx === null) {
    throw new Error('useAgent must be used within AgentProvider');
  }
  return ctx;
}
