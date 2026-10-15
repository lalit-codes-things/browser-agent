import { useCallback, useEffect, useRef, useState } from 'react';
import type {
  ApprovalView,
  HandoffView,
  RuntimeErrorView,
  TaskStatusView,
} from '../bridge/protocol';
import type { AgentPanelState, TaskEventView, TaskState, WaitReason } from '../domain/types';
import {
  isTerminalTaskState,
  isWaitReason,
  taskLifecycleLabel,
  verificationCopy,
} from '../domain/copy';
import { asTaskState } from '../domain/state';
import {
  COMPOSER_OVER_LIMIT_COPY,
  MAX_TASK_CODE_POINTS,
  RUNTIME_NOT_CONNECTED_COPY,
  RUNTIME_NOT_CONNECTED_DETAIL,
  countCodePoints,
} from '../domain/useAgent';

/**
 * The Agent panel is mandatory and permanently visible (Section 0.2).
 *
 * It states capability readiness, task lifecycle, task budget and verification
 * outcome as independent fields and never collapses them into one status. Every
 * value shown comes from a validated runtime event; nothing here is estimated,
 * animated or simulated.
 *
 * Focus contract ([BA-A11Y-002]): when a confirmation appears, focus moves to
 * the safest non-approving option, never to Approve; when it closes, focus
 * returns to the panel so the user is not dropped at the top of the window.
 */

interface Props {
  agent: AgentPanelState;
  runtimeConnected: boolean;
  notice: string | null;
  onSubmit: (text: string) => boolean;
  onPause?: (() => void) | undefined;
  onCancel?: (() => void) | undefined;
  onTakeOver?: (() => void) | undefined;
  onResume?: (() => void) | undefined;
  onExtendBudget?: (() => void) | undefined;
  onFinishWithResults?: (() => void) | undefined;
  onRespondToApproval?: ((actionId: string, approved: boolean) => void) | undefined;
}

function activeState(status: TaskStatusView | null): string | null {
  if (status === null) return null;
  return status.state;
}

function budgetLine(status: TaskStatusView): string {
  const budget = status.budget;
  return [
    `steps ${budget.action_proposals.used}/${budget.action_proposals.limit}`,
    `actions ${budget.dispatched_actions.used}/${budget.dispatched_actions.limit}`,
    `navigations ${budget.navigations.used}/${budget.navigations.limit}`,
    `input tokens ${budget.input_tokens.used}/${budget.input_tokens.limit}`,
    `output tokens ${budget.output_tokens.used}/${budget.output_tokens.limit}`,
  ].join(', ');
}

function duration(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return minutes === 0 ? `${seconds}s` : `${minutes}m ${seconds}s`;
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
  onExtendBudget,
  onFinishWithResults,
  onRespondToApproval,
}: Props) {
  const panelRef = useRef<HTMLElement | null>(null);
  const approvalCancelRef = useRef<HTMLButtonElement>(null);
  const hadApproval = useRef(false);

  const approvalOpen = agent.approval !== null;

  // Focus lands on the non-approving option while a confirmation is pending.
  useEffect(() => {
    if (approvalOpen) {
      approvalCancelRef.current?.focus();
    } else if (hadApproval.current) {
      panelRef.current?.focus();
    }
    hadApproval.current = approvalOpen;
  }, [approvalOpen]);

  const readiness = agent.readiness;
  const state = activeState(agent.status);
  const taskState = state === null ? null : asTaskState(state);

  return (
    <aside className="ba-agent-panel" aria-label="Agent" ref={panelRef} tabIndex={-1}>
      <header className="ba-agent-panel__header">
        <h1 className="ba-agent-panel__heading">Agent</h1>
        <span
          className={`ba-agent-readiness ba-agent-readiness--${readiness.status.toLowerCase()}`}
        >
          {readiness.label}
        </span>
      </header>

      <p className="ba-readiness-detail">
        {runtimeConnected
          ? readiness.message === ''
            ? 'The runtime reported this capability state and no measured detail.'
            : readiness.message
          : `${RUNTIME_NOT_CONNECTED_COPY}. ${RUNTIME_NOT_CONNECTED_DETAIL}`}
      </p>

      {notice !== null && (
        <p className="ba-agent-notice" role="status">
          {notice}
        </p>
      )}

      {agent.error !== null && <ErrorBlock error={agent.error} />}

      {agent.handoff !== null && <HandoffBlock handoff={agent.handoff} />}

      {agent.approval !== null && (
        <ApprovalBlock
          approval={agent.approval}
          cancelRef={approvalCancelRef}
          onRespondToApproval={onRespondToApproval}
        />
      )}

      {agent.status !== null && (
        <TaskStatusBlock
          status={agent.status}
          state={taskState}
          onPause={onPause}
          onCancel={onCancel}
          onTakeOver={onTakeOver}
          onResume={onResume}
          onExtendBudget={onExtendBudget}
          onFinishWithResults={onFinishWithResults}
        />
      )}

      {agent.ended !== null && <EndedBlock ended={agent.ended} />}

      {agent.status === null && agent.ended === null && (
        <Composer onSubmit={onSubmit} locked={agent.composerLocked} />
      )}

      {agent.activity.length > 0 && <ActivityList events={agent.activity} />}

      <AgentDisclosure />
    </aside>
  );
}

/**
 * Persistent disclosure of the local model's input boundary
 * ([BA-TRANSPARENCY-001]). Shown before the first task and afterwards from the
 * same place.
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

function Composer({ onSubmit, locked }: { onSubmit: (text: string) => boolean; locked: boolean }) {
  const [text, setText] = useState('');
  const codePoints = countCodePoints(text);
  const overLimit = codePoints > MAX_TASK_CODE_POINTS;

  const submit = useCallback(
    (event: React.FormEvent) => {
      event.preventDefault();
      if (locked || overLimit || text.trim().length === 0) return;
      // The draft is kept unless the command was actually handed to the runtime.
      if (onSubmit(text)) setText('');
    },
    [locked, onSubmit, overLimit, text],
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
          disabled={overLimit || text.trim().length === 0}
        >
          Submit
        </button>
      </div>
    </form>
  );
}

/**
 * Structured confirmation ([BA-POLICY-002]). The request states the exact
 * origin, destination, target and effect, and the decision is bound to the
 * digest the runtime showed. Cancel is the default focus target.
 */
function ApprovalBlock({
  approval,
  cancelRef,
  onRespondToApproval,
}: {
  approval: ApprovalView;
  cancelRef: React.RefObject<HTMLButtonElement>;
  onRespondToApproval?: ((actionId: string, approved: boolean) => void) | undefined;
}) {
  return (
    <section
      className="ba-wait"
      aria-labelledby="ba-approval-heading"
      data-action-id={approval.action_id}
    >
      <h2 className="ba-approval-label" id="ba-approval-heading">
        {approval.action_label} — your approval is required
      </h2>
      <dl className="ba-approval-detail">
        <dt>Origin</dt>
        <dd>{approval.origin}</dd>
        <dt>Target</dt>
        <dd>{approval.target_label ?? 'The action does not name a specific target.'}</dd>
        <dt>Destination</dt>
        <dd>{approval.destination ?? 'This action does not leave the current page.'}</dd>
        <dt>Why this needs you</dt>
        <dd>{approval.why}</dd>
        <dt>Expected effect</dt>
        <dd>{approval.expected_effect}</dd>
        <dt>Undo</dt>
        <dd>{approval.undo ?? 'No tested compensating action exists for this step.'}</dd>
      </dl>
      {approval.parameters.length > 0 && (
        <ul className="ba-plain-list ba-plain-list--compact">
          {approval.parameters.map(([name, value]) => (
            <li key={name}>
              {name}: {value}
            </li>
          ))}
        </ul>
      )}
      <p className="ba-status ba-status--muted">
        Stops being actionable at{' '}
        {new Date(approval.expires_at_unix_ms).toLocaleTimeString(undefined, {
          hour: '2-digit',
          minute: '2-digit',
          second: '2-digit',
        })}
        . The runtime refuses this decision if the page or target changed after it was shown.
      </p>
      <div className="ba-approval-actions">
        <button
          type="button"
          className="ba-btn ba-btn--primary"
          disabled={onRespondToApproval === undefined}
          onClick={() => onRespondToApproval?.(approval.action_id, true)}
        >
          Approve once
        </button>
        <button
          type="button"
          ref={cancelRef}
          className="ba-btn ba-btn--secondary"
          disabled={onRespondToApproval === undefined}
          onClick={() => onRespondToApproval?.(approval.action_id, false)}
        >
          Cancel
        </button>
      </div>
    </section>
  );
}

/**
 * A step the agent may not perform ([BA-HANDOFF-001]). The user is told what the
 * agent could not do, why, and exactly what they should do instead.
 */
function HandoffBlock({ handoff }: { handoff: HandoffView }) {
  return (
    <section className="ba-wait" aria-labelledby="ba-handoff-heading">
      <h2 className="ba-approval-label" id="ba-handoff-heading">
        {handoff.action_label} — this step is yours
      </h2>
      <dl className="ba-approval-detail">
        <dt>Origin</dt>
        <dd>{handoff.origin}</dd>
        <dt>Target</dt>
        <dd>{handoff.target_label ?? 'No specific target is named.'}</dd>
        <dt>Why the agent stopped</dt>
        <dd>{handoff.reason}</dd>
        <dt>Known consequences</dt>
        <dd>{handoff.consequences}</dd>
        <dt>What to do</dt>
        <dd>{handoff.user_action_hint}</dd>
      </dl>
    </section>
  );
}

/**
 * Four-part error contract ([BA-ERROR-COPY-001]): attempted action, observed
 * event, what is known now, and a safe next action. The diagnostic code is kept
 * inside a collapsed details section.
 */
function ErrorBlock({ error }: { error: RuntimeErrorView }) {
  return (
    <section className="ba-wait ba-wait--error" role="alert" aria-labelledby="ba-error-known">
      <p className="ba-status">{error.attempted}</p>
      <p className="ba-status">{error.observed}</p>
      <p className="ba-status" id="ba-error-known">
        {error.known}
      </p>
      <p className="ba-status">{error.next_action}</p>
      <details className="ba-disclosure">
        <summary className="ba-disclosure-summary">Diagnostic detail</summary>
        <p className="ba-disclosure-body ba-diagnostic">
          Error identifier: {error.error_id}
          {error.rule === null ? '' : ` — blocking rule: ${error.rule}`}
        </p>
      </details>
    </section>
  );
}

function TaskStatusBlock({
  status,
  state,
  onPause,
  onCancel,
  onTakeOver,
  onResume,
  onExtendBudget,
  onFinishWithResults,
}: {
  status: TaskStatusView;
  state: TaskState | null;
  onPause?: (() => void) | undefined;
  onCancel?: (() => void) | undefined;
  onTakeOver?: (() => void) | undefined;
  onResume?: (() => void) | undefined;
  onExtendBudget?: (() => void) | undefined;
  onFinishWithResults?: (() => void) | undefined;
}) {
  const waitReason: WaitReason | null = isWaitReason(status.wait_reason)
    ? status.wait_reason
    : null;
  const label =
    state === null
      ? 'The runtime reported a task state this build does not know, so no action is offered.'
      : taskLifecycleLabel(state, waitReason);
  // The lifecycle label above already fails closed with the canonical invalid
  // copy, so this block only adds the explanation and offers no action.
  const invalidWait = state === 'WAITING_FOR_USER' && waitReason === null;
  const outcome = verificationCopy(status.verification, state ?? 'CREATED');

  return (
    <div className="ba-task-body">
      <div className="ba-task-status">
        <span
          className={`ba-task-state ba-task-state--${(state ?? 'created').toLowerCase()}`}
          role="status"
          aria-live="polite"
        >
          {label}
        </span>
        <span
          className={`ba-verification ${
            outcome.verifiedSuccess ? 'ba-verified-success' : 'ba-verification-muted'
          }`}
        >
          {outcome.label}
        </span>
      </div>

      <dl className="ba-task-figures">
        <dt>Budget used</dt>
        <dd>{budgetLine(status)}</dd>
        <dt>Active time</dt>
        <dd>{duration(status.elapsed_active_ms)}</dd>
        <dt>Waiting time</dt>
        <dd>{duration(status.elapsed_waiting_ms)}</dd>
        <dt>Tabs in task</dt>
        <dd>{status.task_tabs.length === 0 ? 'None reported' : status.task_tabs.join(', ')}</dd>
      </dl>

      {invalidWait && (
        <div className="ba-wait ba-wait--invalid">
          <p className="ba-status ba-status--muted">
            The runtime did not supply a valid wait reason, so no action is offered. This state
            fails closed.
          </p>
        </div>
      )}

      <div className="ba-approval-actions">
        {onPause !== undefined &&
          (state === 'EXECUTING' || state === 'OBSERVING' || state === 'PROPOSING') && (
            <button type="button" className="ba-btn ba-btn--secondary" onClick={onPause}>
              Pause
            </button>
          )}
        {onResume !== undefined && (state === 'PAUSED' || state === 'USER_TAKEOVER') && (
          <button type="button" className="ba-btn ba-btn--primary" onClick={onResume}>
            Resume
          </button>
        )}
        {onTakeOver !== undefined &&
          state !== null &&
          state !== 'USER_TAKEOVER' &&
          !isTerminalTaskState(state) && (
            <button type="button" className="ba-btn ba-btn--secondary" onClick={onTakeOver}>
              Take over
            </button>
          )}
        {/* The pending approval block above already owns the cancel decision,
         * so no second control for the same action is rendered here. */}
        {onExtendBudget !== undefined && waitReason === 'TASK_BUDGET' && (
          <button type="button" className="ba-btn ba-btn--secondary" onClick={onExtendBudget}>
            Extend task budget
          </button>
        )}
        {onFinishWithResults !== undefined && waitReason === 'TASK_BUDGET' && (
          <button type="button" className="ba-btn ba-btn--secondary" onClick={onFinishWithResults}>
            Finish with current results
          </button>
        )}
        {onCancel !== undefined && (state === null || !isTerminalTaskState(state)) && (
          <button type="button" className="ba-btn ba-btn--secondary" onClick={onCancel}>
            Cancel task
          </button>
        )}
      </div>
    </div>
  );
}

function EndedBlock({ ended }: { ended: AgentPanelState['ended'] }) {
  if (ended === null) return null;
  const state = asTaskState(ended.state);
  const outcome = verificationCopy(ended.verification, state ?? 'COMPLETED');
  return (
    <div className="ba-task-body">
      <div className="ba-task-status">
        <span className={`ba-task-state ba-task-state--${(state ?? 'completed').toLowerCase()}`}>
          {ended.state_label}
        </span>
        <span
          className={`ba-verification ${
            outcome.verifiedSuccess ? 'ba-verified-success' : 'ba-verification-muted'
          }`}
        >
          {ended.verification_label ?? outcome.label}
        </span>
      </div>
      <p className="ba-status">{ended.detail}</p>
    </div>
  );
}

function ActivityList({ events }: { events: readonly TaskEventView[] }) {
  return (
    <ol className="ba-activity">
      {events.map((event) => (
        <li
          key={`${event.task_id}-${event.revision}-${event.sequence}`}
          className="ba-activity-item"
        >
          <time dateTime={new Date(event.at_unix_ms).toISOString()}>
            {new Date(event.at_unix_ms).toLocaleTimeString(undefined, {
              hour: '2-digit',
              minute: '2-digit',
              second: '2-digit',
            })}
          </time>
          <p>{event.label}</p>
          {event.detail !== '' && <p className="ba-status ba-status--muted">{event.detail}</p>}
        </li>
      ))}
    </ol>
  );
}
