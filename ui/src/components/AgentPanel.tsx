import React, { useCallback } from 'react';
import {
  AgentPanelState,
  TaskSummary,
  ApprovalDetails,
  TaskEvent,
  TaskState,
  WaitReason,
  VerificationOutcome,
} from '../domain/types';

const TASK_COPY: Record<TaskState, string> = {
  CREATED: 'Task received',
  NORMALIZING: 'Preparing task',
  OBSERVING: 'Inspecting page',
  PROPOSING: 'Preparing next step',
  POLICY_CHECK: 'Checking permission',
  EXECUTING: 'Performing action',
  VERIFYING: 'Checking result',
  WAITING_FOR_USER: 'Waiting for your approval',
  USER_TAKEOVER: "You're in control",
  PAUSED: 'Paused',
  BLOCKED: 'Blocked by policy',
  FAILED: 'Task failed',
  CANCELLED: 'Task cancelled',
  COMPLETED: 'Task ended',
};

const WAIT_COPY: Record<WaitReason, { label: string; submit: boolean }> = {
  ACTION_APPROVAL: { label: 'Waiting for your approval', submit: true },
  USER_INPUT: { label: 'Needs your input', submit: false },
  AUTHENTICATION: { label: 'Waiting for you to sign in', submit: false },
  TASK_BUDGET: { label: 'Task budget reached', submit: true },
};

function outcomeCopy(outcome: VerificationOutcome): { label: string; class: string } {
  switch (outcome) {
    case 'VERIFIED_SUCCESS':
      return { label: 'Verified success', class: 'ba-verified-success' };
    case 'LIKELY_SUCCESS':
      return {
        label: 'Likely succeeded - not fully verified',
        class: 'ba-verification-muted',
      };
    case 'UNKNOWN':
      return { label: 'Result unknown', class: 'ba-verification-muted' };
    case 'LIKELY_FAILURE':
      return { label: 'Likely did not succeed', class: 'ba-verification-muted' };
    case 'VERIFIED_FAILURE':
      return { label: 'Verified failure', class: 'ba-verified-failure' };
  }
}

interface Props {
  agent: AgentPanelState;
  onSubmit: (text: string) => Promise<string>;
}

export function AgentPanel({ agent, onSubmit }: Props) {
  const approval = agent.approval;
  const waitCopy: { label: string; submit: boolean } | null =
    approval?.action != null ? WAIT_COPY.ACTION_APPROVAL : null;

  return (
    <aside className="ba-agent-panel">
      <header className="ba-agent-panel__header">
        <h1 className="ba-agent-panel__heading">Agent</h1>
        <span className={`ba-agent-readiness ba-agent-readiness--idle`}>
          {agent.readiness.message}
        </span>
      </header>

      {agent.task ? (
        <TaskBody
          task={agent.task}
          approval={approval}
          waitCopy={waitCopy}
          onPause={agent.composerLocked ? undefined : () => undefined}
          onCancel={agent.composerLocked ? undefined : () => undefined}
          onTakeOver={agent.composerLocked ? undefined : () => undefined}
          onResume={agent.composerLocked ? undefined : () => undefined}
          onRespondToApproval={agent.composerLocked ? undefined : () => undefined}
        />
      ) : (
        <Composer onSubmit={onSubmit} locked={agent.composerLocked} />
      )}

      {agent.task && agent.task.events.length > 0 && <ActivityList events={agent.task.events} />}
    </aside>
  );
}

function Composer({
  onSubmit,
  locked,
}: {
  onSubmit: (text: string) => Promise<string>;
  locked: boolean;
}) {
  const [text, setText] = React.useState('');

  const submit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      if (!text.trim() || locked) return;
      void onSubmit(text);
      setText('');
    },
    [text, locked, onSubmit],
  );

  if (locked) {
    return (
      <div className="ba-composer ba-composer--locked">
        <p>One Agent task is already running. Finish or cancel it before starting another.</p>
      </div>
    );
  }

  return (
    <form className="ba-composer" onSubmit={submit}>
      <label className="ba-composer-label" htmlFor="ba-composer">
        Task
      </label>
      <textarea
        id="ba-composer"
        className="ba-composer-input"
        value={text}
        maxLength={8000}
        onChange={(e) => setText(e.target.value)}
        placeholder="Describe a task you want the agent to perform in the browser."
      />
      <div className="ba-composer-actions">
        <span className="ba-composer-counter">{text.length}/8000</span>
        <button type="submit" className="ba-btn ba-btn-primary" disabled={!text.trim()}>
          Submit
        </button>
      </div>
    </form>
  );
}

interface TaskBodyProps {
  task: TaskSummary;
  approval: ApprovalDetails | null;
  waitCopy: { label: string; submit: boolean } | null;
  onPause?: () => void | undefined;
  onCancel?: () => void | undefined;
  onTakeOver?: () => void | undefined;
  onResume?: () => void | undefined;
  onRespondToApproval?: () => void | undefined;
}

function TaskBody({
  task,
  approval,
  waitCopy,
  onPause,
  onCancel,
  onTakeOver,
  onResume,
  onRespondToApproval,
}: TaskBodyProps) {
  const stateLabel = TASK_COPY[task.state];
  const outcome = outcomeCopy(task.verification ?? 'UNKNOWN');

  return (
    <div className="ba-task-body">
      <div className="ba-task-status">
        <span className={`ba-task-state ba-task-state--${task.state.toLowerCase()}`}>
          {stateLabel}
        </span>
        <span className="ba-verification">{outcome.label}</span>
      </div>

      {task.waitReason != null && waitCopy && (
        <div className="ba-wait">
          <p>{waitCopy.label}</p>
          {waitCopy.submit &&
            task.state === 'WAITING_FOR_USER' &&
            approval &&
            onRespondToApproval && (
              <div className="ba-approval">
                <p className="ba-approval-label">Approve this bounded action</p>
                <p className="ba-approval-detail">
                  {approval.action.actionType} on {approval.action.targetOrigin}
                </p>
                <div className="ba-approval-actions">
                  <button className="ba-btn ba-btn-primary" onClick={onRespondToApproval}>
                    Approve action
                  </button>
                  <button className="ba-btn ba-btn-secondary" onClick={onRespondToApproval}>
                    Cancel
                  </button>
                </div>
              </div>
            )}
          {!waitCopy.submit && task.state === 'WAITING_FOR_USER' && (
            <div className="ba-approval">
              <p>{waitCopy.label}</p>
              {onTakeOver && (
                <button className="ba-btn ba-btn-primary" onClick={onTakeOver}>
                  Take over
                </button>
              )}
              {onCancel && (
                <button className="ba-btn ba-btn-secondary" onClick={onCancel}>
                  Cancel task
                </button>
              )}
            </div>
          )}
        </div>
      )}

      {onPause && task.state === 'EXECUTING' && (
        <button className="ba-btn ba-btn-secondary" onClick={onPause}>
          Pause
        </button>
      )}
      {onCancel && (task.state === 'PAUSED' || task.state === 'OBSERVING') && (
        <button className="ba-btn ba-btn-secondary" onClick={onCancel}>
          Cancel task
        </button>
      )}
      {onResume && (task.state === 'PAUSED' || task.state === 'USER_TAKEOVER') && (
        <button className="ba-btn ba-btn-primary" onClick={onResume}>
          Resume
        </button>
      )}
      {onTakeOver && (task.state === 'PROPOSING' || task.state === 'POLICY_CHECK') && (
        <button className="ba-btn ba-btn-secondary" onClick={onTakeOver}>
          Take over
        </button>
      )}
    </div>
  );
}

function ActivityList({ events }: { events: readonly TaskEvent[] }) {
  return (
    <ol className="ba-activity">
      {events.map((e) => (
        <li key={`${e.taskId}-${e.sequence}`} className="ba-activity-item">
          <time dateTime={new Date(e.timestamp).toISOString()}>{formatClock(e.timestamp)}</time>
          <p>{e.actionLabel ?? e.state}</p>
        </li>
      ))}
    </ol>
  );
}

function formatClock(ts: number): string {
  const d = new Date(ts);
  return d.toLocaleTimeString(undefined, {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}
