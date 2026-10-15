import { createContext, useCallback, useContext, useMemo, useState } from 'react';
import type { ReactNode } from 'react';
import { commands } from '../bridge/client';
import { panelStateFromSnapshot } from './state';
import { useBrowser } from './useBrowser';
import { useRuntime } from './useRuntime';
import type { AgentPanelState } from './types';

/**
 * Agent panel state, projected from validated runtime events.
 *
 * Nothing in this module may synthesize a task state, advance a lifecycle with a
 * timer or report verification success: React renders authoritative runtime
 * state and cannot invent transitions (Section 12.2). When no host is attached
 * the panel reports that honestly instead of demonstrating a simulated run.
 */

/** Composer ceiling in Unicode code points ([BA-COMPOSER-001]). */
export const MAX_TASK_CODE_POINTS = 8000;

/** Real count of Unicode code points, not UTF-16 units. */
export function countCodePoints(text: string): number {
  return Array.from(text).length;
}

export const RUNTIME_NOT_CONNECTED_COPY = 'Agent runtime not connected';

export const RUNTIME_NOT_CONNECTED_DETAIL =
  'The managed browser host is not attached in this build, so no model can be installed or loaded and no task can start.';

export const TASK_NOT_STARTED_COPY =
  'No task was started: the agent runtime is not connected, so the request was not sent anywhere.';

/** No managed window is attached, so the runtime has no origin to bind policy to. */
export const NO_MANAGED_WINDOW_COPY =
  'No task was started: no managed browser window is attached, so there is no page for the agent to work in.';

export const COMPOSER_OVER_LIMIT_COPY =
  'This task exceeds the 8,000 code point limit. Shorten it before submitting; the text is not truncated.';

export const TASK_ALREADY_RUNNING_COPY =
  'One agent task is already running. Finish or cancel it before starting another.';

export interface AgentContextValue {
  state: AgentPanelState;
  /** True only when a real managed host has attached its channel. */
  runtimeConnected: boolean;
  /** Honest, non-simulated status message for the panel. */
  notice: string | null;
  submitTask: (text: string) => boolean;
  pause: () => void;
  cancel: () => void;
  takeOver: () => void;
  resume: () => void;
  extendBudget: () => void;
  finishWithResults: () => void;
  /** Approves or refuses the pending action, bound to what was displayed. */
  respondToApproval: (actionId: string, approved: boolean) => void;
}

const AgentContext = createContext<AgentContextValue | null>(null);

export function AgentProvider({ children }: { children: ReactNode }) {
  const runtime = useRuntime();
  const browser = useBrowser();
  const runtimeConnected = runtime.runtimeConnected;
  const client = runtime.client;
  const state = useMemo(() => panelStateFromSnapshot(runtime.snapshot), [runtime.snapshot]);
  const [notice, setNotice] = useState<string | null>(null);

  const submitTask = useCallback(
    (text: string): boolean => {
      const trimmed = text.trim();
      if (trimmed.length === 0) return false;

      if (countCodePoints(trimmed) > MAX_TASK_CODE_POINTS) {
        setNotice(COMPOSER_OVER_LIMIT_COPY);
        return false;
      }

      // One active autonomous task process-wide ([BA-WIN-002]). A task already
      // in flight must be resolved before another is submitted.
      if (state.composerLocked) {
        setNotice(TASK_ALREADY_RUNNING_COPY);
        return false;
      }

      if (!runtimeConnected || client === null) {
        // Never fabricate a task or a lifecycle for a runtime that is absent.
        setNotice(TASK_NOT_STARTED_COPY);
        return false;
      }

      const window = browser.state.windowId;
      const tab = browser.state.activeTabId;
      if (window === null || tab === null) {
        setNotice(NO_MANAGED_WINDOW_COPY);
        return false;
      }

      setNotice(null);
      client.send(commands.submitTask(trimmed, window, tab));
      return true;
    },
    [
      browser.state.activeTabId,
      browser.state.windowId,
      client,
      runtimeConnected,
      state.composerLocked,
    ],
  );

  const send = useCallback(
    (command: ReturnType<typeof commands.pause>) => {
      client?.send(command);
    },
    [client],
  );

  const respondToApproval = useCallback(
    (actionId: string, approved: boolean) => {
      const approval = state.approval;
      // A decision is only valid for the pending request it was shown for, and
      // the approval is single-use and expires in the runtime.
      if (approval?.action_id !== actionId) return;
      send(
        approved
          ? commands.approveAction(
              approval.action_id,
              approval.digest,
              approval.origin,
              approval.epoch,
              approval.frame,
            )
          : commands.cancelApproval(),
      );
    },
    [send, state.approval],
  );

  const value = useMemo<AgentContextValue>(
    () => ({
      state,
      runtimeConnected,
      // Local notices only: a runtime error is rendered by the panel from the
      // event itself, and repeating it as a notice would state it twice.
      notice,
      submitTask,
      pause: () => {
        send(commands.pause());
      },
      cancel: () => {
        send(commands.cancelTask());
      },
      takeOver: () => {
        send(commands.takeOver());
      },
      resume: () => {
        send(commands.resume());
      },
      extendBudget: () => {
        send(commands.extendBudget());
      },
      finishWithResults: () => {
        send(commands.finishWithResults());
      },
      respondToApproval,
    }),
    [notice, respondToApproval, runtimeConnected, send, state, submitTask],
  );

  return <AgentContext.Provider value={value}>{children}</AgentContext.Provider>;
}

export function useAgent(): AgentContextValue {
  const ctx = useContext(AgentContext);
  if (ctx === null) {
    throw new Error('useAgent must be used within AgentProvider');
  }
  return ctx;
}
