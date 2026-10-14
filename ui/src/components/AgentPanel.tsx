import { useCallback, useState } from 'react';
import type {
  AgentPanelState,
  ApprovalDetails,
  TaskEvent,
  TaskSummary,
  WaitReason,
} from '../domain/types';
import {
  READINESS_COPY,
  VERIFICATION_NOT_RUN_COPY,
  isWaitReason,
  taskLifecycleLabel,
  verificationCopy,
} from '../domain/copy';
import {
  COMPOSER_OVER_LIMIT_COPY,
  MAX_TASK_CODE_POINTS,
  RUNTIME_NOT_CONNECTED_DETAIL,
  RUNTIME_NOT_CONNECTED_COPY,
  countCodePoints,
} from '../domain/useAgent';

/**
 * The Agent panel is mandatory and permanently visible (Section 0.2). It states
 * capability readiness, task lifecycle and verification outcome as three
 * independent fields and never collapses them into one status.
 */

interface Props {
  agent: AgentPanelState;
  runtimeConnected: boolean;
  notice: string | null;
  onSubmit: (text: string) => Promise<string>;
  onPause?: (() => void) | undefined;
  onCancel?: (() => void) | undefined;
  onTakeOver?: (() => void) | undefined;
  onResume?: (() => void) | undefined;
  onRespondToApproval?: ((actionId: string, approved: boolean) => void) | undefined;
}

export function AgentPanel({
  agent,
  runtimeConnected,
  notice,
  onSubmit,
  onPause,
  onCancel,
  onTakeOver,
  onResume,
  onRespondToApproval,
}: Props) {
  const readiness = agent.readiness;

  return (
    <aside className="ba-agent-panel" aria-label="Agent">
      <header className="ba-agent-panel__header">
        <h1 className="ba-agent-panel__heading">Agent</h1>
        <span
          className={`ba-agent-readiness ba-agent-readiness--${readiness.status.toLowerCase()}`}
        >
          {READINESS_COPY[readiness.status]}
        </span>
      </header>

      <p className="ba-readiness-detail">
        {runtimeConnected
          ? readiness.message
          : `${RUNTIME_NOT_CONNECTED_COPY}. ${RUNTIME_NOT_CONNECTED_DETAIL}`}
      </p>

      {notice !== null && (
        <p className="ba-agent-notice" role="status">
          {notice}
        </p>
      )}

      {agent.task ? (
        <TaskBody
          task={agent.task}
          approval={agent.approval}
          onPause={onPause}
          onCancel={onCancel}
          onTakeOver={onTakeOver}
          onResume={onResume}
          onRespondToApproval={onRespondToApproval}
        />
      ) : (
        <Composer onSubmit={onSubmit} locked={agent.composerLocked} />
      )}

      {agent.task && agent.task.events.length > 0 && <ActivityList events={agent.task.events} />}

      <AgentDisclosure />
    </aside>
  );
}

/**
 * Persistent disclosure of the local model's input boundary ([BA-TRANSPARENCY-001]).
 * Shown before the first task and afterwards from the same place.
 */
function AgentDisclosure() {
  return (
    <details className="ba-disclosure">
      <summary className="ba-disclosure-summary">What the agent will see</summary>
      <p className="ba-disclosure-body">
        A local model may receive the task text, the current page origin and title, bounded visible
        text for the elements it is working with, accessibility labels and roles, relevant
        relationships, and a redacted summary of earlier task steps.
      </p>
      <p className="ba-disclosure-body">
        It does not receive vault values, cookies, session tokens, authorization headers, raw
        scripts, raw network bodies or unrestricted raw HTML.
      </p>
      <p className="ba-disclosure-body">
        A website can still receive any value you authorize the agent to submit. The model runs
        locally; no cloud model endpoint or API key is used.
      </p>
    </details>
  );
}

function Composer({
  onSubmit,
  locked,
}: {
  onSubmit: (text: string) => Promise<string>;
  locked: boolean;
}) {
  const [text, setText] = useState('');
  const [busy, setBusy] = useState(false);
  const codePoints = countCodePoints(text);
  const overLimit = codePoints > MAX_TASK_CODE_POINTS;

  const submit = useCallback(
    async (event: React.FormEvent) => {
      event.preventDefault();
      if (locked || busy || overLimit || text.trim().length === 0) return;
      setBusy(true);
      try {
        const taskId = await onSubmit(text);
        // Draft is cleared only after a task was actually created.
        if (taskId !== '') setText('');
      } finally {
        setBusy(false);
      }
    },
    [busy, locked, onSubmit, overLimit, text],
  );

  if (locked) {
    return (
      <div className="ba-composer ba-composer--locked">
        <p>One Agent task is already running. Finish or cancel it before starting another.</p>
      </div>
    );
  }

  return (
    <form
      className="ba-composer"
      onSubmit={(event) => {
        void submit(event);
      }}
    >
      <label className="ba-composer-label" htmlFor="ba-composer">
        Task
      </label>
      <textarea
        id="ba-composer"
        className="ba-composer-input"
        value={text}
        onChange={(event) => setText(event.target.value)}
        placeholder="Describe a task you want the agent to perform in the browser."
        aria-describedby="ba-composer-counter"
      />
      <p
        id="ba-composer-counter"
        className={`ba-composer-counter${overLimit ? ' ba-composer-counter--over' : ''}`}
        role={overLimit ? 'alert' : undefined}
      >
        {overLimit ? COMPOSER_OVER_LIMIT_COPY : `${codePoints}/${MAX_TASK_CODE_POINTS} code points`}
      </p>
      <div className="ba-composer-actions">
        <button
          type="submit"
          className="ba-btn ba-btn--primary"
          disabled={busy || overLimit || text.trim().length === 0}
        >
          Submit
        </button>
      </div>
    </form>
  );
}

interface TaskBodyProps {
  task: TaskSummary;
  approval: ApprovalDetails | null;
  onPause?: (() => void) | undefined;
  onCancel?: (() => void) | undefined;
  onTakeOver?: (() => void) | undefined;
  onResume?: (() => void) | undefined;
  onRespondToApproval?: ((actionId: string, approved: boolean) => void) | undefined;
}

function TaskBody({
  task,
  approval,
  onPause,
  onCancel,
  onTakeOver,
  onResume,
  onRespondToApproval,
}: TaskBodyProps) {
  // A `WAITING_FOR_USER` state without a typed reason is schema-invalid and
  // must fail closed instead of rendering any action ([BA-STATE-002]).
  const waitReason = isWaitReason(task.waitReason) ? task.waitReason : null;
  const lifecycle = taskLifecycleLabel(task.state, waitReason);
  const outcome = verificationCopy(task.verification, task.state);

  return (
    <div className="ba-task-body">
      <div className="ba-task-status">
        <span className={`ba-task-state ba-task-state--${task.state.toLowerCase()}`}>
          {lifecycle}
        </span>
        <span
          className={`ba-verification ${
            outcome.verifiedSuccess ? 'ba-verified-success' : 'ba-verification-muted'
          }`}
        >
          {outcome.label}
        </span>
      </div>

      {task.state === 'WAITING_FOR_USER' && waitReason === null && (
        <div className="ba-wait ba-wait--invalid">
          <p>{lifecycle}</p>
          <p className="ba-status ba-status--muted">
            The runtime did not supply a valid wait reason, so no action is offered. This state
            fails closed.
          </p>
        </div>
      )}

      {task.state === 'WAITING_FOR_USER' && waitReason !== null && (
        <WaitControls
          reason={waitReason}
          approval={approval}
          onCancel={onCancel}
          onTakeOver={onTakeOver}
          onRespondToApproval={onRespondToApproval}
        />
      )}

      {task.state === 'EXECUTING' && onPause && (
        <button type="button" className="ba-btn ba-btn--secondary" onClick={onPause}>
          Pause
        </button>
      )}

      {(task.state === 'PAUSED' ||
        task.state === 'USER_TAKEOVER' ||
        task.state === 'OBSERVING') && (
        <div className="ba-approval-actions">
          {onResume && (task.state === 'PAUSED' || task.state === 'USER_TAKEOVER') && (
            <button type="button" className="ba-btn ba-btn--primary" onClick={onResume}>
              Resume
            </button>
          )}
          {onCancel && (
            <button type="button" className="ba-btn ba-btn--secondary" onClick={onCancel}>
              Cancel task
            </button>
          )}
        </div>
      )}

      {(task.state === 'PROPOSING' || task.state === 'POLICY_CHECK') && onTakeOver && (
        <button type="button" className="ba-btn ba-btn--secondary" onClick={onTakeOver}>
          Take over
        </button>
      )}

      {task.state === 'BLOCKED' && (
        <p className="ba-status ba-status--muted">{VERIFICATION_NOT_RUN_COPY}</p>
      )}
    </div>
  );
}

/**
 * Controls for `WAITING_FOR_USER`, resolved from the typed wait reason so the
 * approval case and the control-handover case can never share one ambiguous
 * continue action (Sections 2.9.1, 2.9.2, 12.2).
 *
 * `TASK_BUDGET` also specifies `Finish with current results` and
 * `Extend task budget`. Those require runtime operations that do not exist in
 * the bridge yet, so only the operations that do exist are rendered here; the
 * remaining two must ship with their operations rather than as dead controls.
 */
function WaitControls({
  reason,
  approval,
  onCancel,
  onTakeOver,
  onRespondToApproval,
}: {
  reason: WaitReason;
  approval: ApprovalDetails | null;
  onCancel?: (() => void) | undefined;
  onTakeOver?: (() => void) | undefined;
  onRespondToApproval?: ((actionId: string, approved: boolean) => void) | undefined;
}) {
  if (reason === 'ACTION_APPROVAL') {
    return (
      <div className="ba-wait">
        <p className="ba-approval-label">Approve this bounded action</p>
        {approval === null ? (
          <p className="ba-status ba-status--muted">
            The runtime is waiting for approval but has not supplied the pending action, so no
            approval control is offered.
          </p>
        ) : (
          <>
            <p className="ba-approval-detail">
              {approval.action.actionType} on {approval.action.targetOrigin}
            </p>
            <div className="ba-approval-actions">
              {onRespondToApproval && (
                <>
                  <button
                    type="button"
                    className="ba-btn ba-btn--primary"
                    onClick={() => onRespondToApproval(approval.actionId, true)}
                  >
                    Approve action
                  </button>
                  <button
                    type="button"
                    className="ba-btn ba-btn--secondary"
                    onClick={() => onRespondToApproval(approval.actionId, false)}
                  >
                    Cancel
                  </button>
                </>
              )}
            </div>
          </>
        )}
      </div>
    );
  }

  return (
    <div className="ba-wait">
      {reason === 'USER_INPUT' && (
        <p className="ba-approval-detail">
          The agent needs an answer before it can continue. No action is dispatched while it waits.
        </p>
      )}
      {reason === 'AUTHENTICATION' && (
        <p className="ba-approval-detail">
          Sign in yourself in the browser. The agent does not read credentials from page text and
          does not continue until you are signed in.
        </p>
      )}
      {reason === 'TASK_BUDGET' && (
        <p className="ba-approval-detail">
          A task budget ceiling was reached. Cancelling stops the task; budget values are reported
          by the runtime and are not estimated here.
        </p>
      )}
      <div className="ba-approval-actions">
        {onTakeOver && (
          <button type="button" className="ba-btn ba-btn--primary" onClick={onTakeOver}>
            Take over
          </button>
        )}
        {onCancel && (
          <button type="button" className="ba-btn ba-btn--secondary" onClick={onCancel}>
            Cancel task
          </button>
        )}
      </div>
    </div>
  );
}

function ActivityList({ events }: { events: readonly TaskEvent[] }) {
  return (
    <ol className="ba-activity">
      {events.map((event) => (
        <li
          key={`${event.taskId}-${event.revision}-${event.sequence}`}
          className="ba-activity-item"
        >
          <time dateTime={new Date(event.timestamp).toISOString()}>
            {formatClock(event.timestamp)}
          </time>
          <p>{event.actionLabel ?? event.state}</p>
        </li>
      ))}
    </ol>
  );
}

function formatClock(ts: number): string {
  return new Date(ts).toLocaleTimeString(undefined, {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}
