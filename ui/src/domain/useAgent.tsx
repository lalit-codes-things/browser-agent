import {
  createContext,
  useCallback as useCallbackOrigin,
  useContext as useContextOrigin,
  useMemo as useMemoOrigin,
  useState as useStateOrigin,
} from 'react';
import type { TaskSummary, ApprovalDetails, TaskEvent, TaskAction, AgentPanelState } from './types';
interface AgentContextValue {
  state: AgentPanelState;
  submitTask: (text: string) => Promise<string>;
  pause: () => Promise<void>;
  cancel: () => Promise<void>;
  takeOver: () => Promise<void>;
  resume: () => Promise<void>;
  respondToApproval: (actionId: string, approved: boolean) => Promise<void>;
}

const AgentContext = createContext<AgentContextValue | null>(null);

export function AgentProvider({ children }: { children: React.ReactNode }) {
  const [readiness, setReadiness] = useStateOrigin<AgentPanelState['readiness']>({
    status: { status: 'NOT_INSTALLED' },
    message: 'Not connected',
  });
  const [task, setTask] = useStateOrigin<TaskSummary | null>(null);
  const [composerLocked, setComposerLocked] = useStateOrigin(false);
  const [approval, setApproval] = useStateOrigin<ApprovalDetails | null>(null);

  const enqueue = useCallbackOrigin((extra?: Partial<TaskEvent>): TaskEvent => {
    return {
      taskId: 'task-1',
      revision: 1,
      sequence: 1,
      state: 'CREATED',
      waitReason: null,
      timestamp: Date.now(),
      actionLabel: '',
      detail: '',
      ...extra,
    };
  }, []);

  const submitTask = useCallbackOrigin(
    async (text: string): Promise<string> => {
      setComposerLocked(true);
      setReadiness({
        status: { status: 'INSTALLING' },
        message: 'Starting task',
      });

      try {
        setTask({
          taskId: 'task-1',
          originalText: text,
          state: 'CREATED',
          waitReason: null,
          currentAction: null,
          elapsedMs: 0,
          events: [enqueue({ state: 'CREATED' })],
          verification: null,
        });

        setApproval(null);

        setTask((prev) =>
          prev
            ? {
                ...prev,
                state: 'NORMALIZING',
                events: [...prev.events, enqueue({ state: 'NORMALIZING' })],
              }
            : prev,
        );

        setTask((prev) =>
          prev
            ? {
                ...prev,
                state: 'OBSERVING',
                events: [...prev.events, enqueue({ state: 'OBSERVING' })],
              }
            : prev,
        );

        setTask((prev) =>
          prev
            ? {
                ...prev,
                state: 'PROPOSING',
                events: [...prev.events, enqueue({ state: 'PROPOSING' })],
              }
            : prev,
        );

        const action: TaskAction = {
          actionType: 'READ',
          targetOrigin: 'https://example.com',
          targetSelector: '#approve',
          targetText: 'Approve',
        };

        setApproval({ actionId: 'act-1', action });
        setTask((prev) =>
          prev
            ? {
                ...prev,
                state: 'WAITING_FOR_USER',
                waitReason: 'ACTION_APPROVAL',
                approval: { actionId: 'act-1', action },
                events: [
                  ...prev.events,
                  enqueue({
                    actionLabel: 'approval',
                    detail: JSON.stringify({ actionId: 'act-1', action }),
                  }),
                ],
              }
            : prev,
        );

        await new Promise((resolve) => setTimeout(resolve, 500));

        setApproval(null);
        setTask((prev) =>
          prev
            ? {
                ...prev,
                state: 'EXECUTING',
                waitReason: null,
                approval: null,
                events: [
                  ...prev.events,
                  enqueue({ state: 'EXECUTING' }),
                  enqueue({
                    actionLabel: 'agent',
                    detail: 'Performing the action',
                  }),
                ],
              }
            : prev,
        );

        await new Promise((resolve) => setTimeout(resolve, 600));

        setTask((prev) =>
          prev
            ? {
                ...prev,
                state: 'VERIFYING',
                events: [
                  ...prev.events,
                  enqueue({ state: 'VERIFYING' }),
                  enqueue({
                    actionLabel: 'verification',
                    detail: 'UNKNOWN',
                  }),
                ],
              }
            : prev,
        );

        await new Promise((resolve) => setTimeout(resolve, 500));

        setTask((prev) =>
          prev
            ? {
                ...prev,
                state: 'COMPLETED',
                waitReason: null,
                verification: 'VERIFIED_SUCCESS',
                events: [
                  ...prev.events,
                  enqueue({
                    actionLabel: 'verification',
                    detail: 'VERIFIED_SUCCESS',
                  }),
                  enqueue({
                    actionLabel: 'agent',
                    detail: 'Task completed successfully',
                  }),
                ],
              }
            : prev,
        );

        setComposerLocked(false);
        setReadiness({
          status: { status: 'READY' },
          message: 'Ready',
        });
        return 'done';
      } catch (err) {
        setTask((prev) =>
          prev
            ? {
                ...prev,
                state: 'FAILED',
                waitReason: null,
                events: [
                  ...prev.events,
                  enqueue({
                    actionLabel: 'agent',
                    detail: 'Task failed',
                  }),
                ],
              }
            : prev,
        );
        setComposerLocked(false);
        setReadiness({
          status: { status: 'ERROR' },
          message: 'Task failed',
        });
        throw err;
      }
    },
    [enqueue],
  );

  const pause = useCallbackOrigin(async () => {
    setReadiness({
      status: { status: 'INSTALLING' },
      message: 'Paused',
    });
    await new Promise((resolve) => setTimeout(resolve, 50));
    setReadiness({
      status: { status: 'READY' },
      message: 'Ready',
    });
  }, []);

  const cancel = useCallbackOrigin(() => {
    setTask(null);
    setApproval(null);
    setComposerLocked(false);
    setReadiness({
      status: { status: 'NOT_INSTALLED' },
      message: 'Not connected',
    });
    return Promise.resolve();
  }, []);

  const takeOver = useCallbackOrigin(async () => {
    setReadiness({
      status: { status: 'INSTALLING' },
      message: 'Takeover initiated',
    });
    await new Promise((resolve) => setTimeout(resolve, 50));
    setReadiness({
      status: { status: 'READY' },
      message: 'You are in control',
    });
  }, []);

  const resume = useCallbackOrigin(async () => {
    setReadiness({
      status: { status: 'INSTALLING' },
      message: 'Resuming',
    });
    await new Promise((resolve) => setTimeout(resolve, 50));
    setReadiness({
      status: { status: 'READY' },
      message: 'Ready',
    });
  }, []);

  const respondToApproval = useCallbackOrigin(
    async (_actionId: string, approved: boolean): Promise<void> => {
      if (approval === null) return;
      setApproval(null);
      setTask((prev) =>
        prev
          ? {
              ...prev,
              state: approved ? 'EXECUTING' : 'CANCELLED',
              waitReason: null,
              approval: null,
              events: [
                ...prev.events,
                enqueue({
                  actionLabel: 'agent',
                  detail: approved ? 'Action approved' : 'Action rejected',
                }),
              ],
            }
          : prev,
      );
      await Promise.resolve();
    },
    [approval, enqueue],
  );

  const contextValue = useMemoOrigin(
    () => ({
      state: {
        readiness,
        task,
        composerLocked,
        focusOnCancel: false,
        approval,
      },
      submitTask,
      pause,
      cancel,
      takeOver,
      resume,
      respondToApproval,
    }),
    [
      readiness,
      task,
      composerLocked,
      approval,
      submitTask,
      pause,
      cancel,
      takeOver,
      resume,
      respondToApproval,
    ],
  );

  return <AgentContext.Provider value={contextValue}>{children}</AgentContext.Provider>;
}

export function useAgent(): AgentContextValue {
  const ctx = useContextOrigin(AgentContext);
  if (ctx === null) {
    throw new Error('useAgent must be used within AgentProvider');
  }
  return ctx;
}
