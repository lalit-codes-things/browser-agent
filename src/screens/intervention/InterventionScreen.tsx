import { memo, useCallback, useState } from "react";
import type { AppState } from "../../state/app";
import {
  abortTask,
  confirmAuthorization,
  denyAuthorization,
  endHumanHandoff,
} from "../../ipc/client";
import { formatIpcError } from "../../ipc/errors";

// Intervention Screen (MASTER PROMPT §8, DESIGN.md):
// Surface for all backend-issued human checkpoints:
//   - Authorization confirmation (CAPTCHA, 2FA, dangerous actions)
//   - Human takeover active (payment handoff, manual interaction)
//   - Denial/expiry notices
// The backend owns every fact shown here. Frontend never fabricates authorization state.

interface ActionButtonProps {
  onClick: () => void;
  disabled: boolean;
  label: string;
  variant: "primary" | "danger" | "muted";
}

function ActionButton({ onClick, disabled, label, variant }: ActionButtonProps) {
  const variantCls =
    variant === "danger"
      ? "border-sem-violation text-sem-violation hover:bg-bg-3"
      : variant === "primary"
        ? "border-line-strong text-primary hover:bg-surface-3"
        : "border-line text-secondary hover:bg-surface-2";
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className={`
        border-1px bg-surface-1 px-5 py-2
        text-[11px] font-semibold uppercase tracking-wider
        disabled:cursor-not-allowed disabled:border-line disabled:text-muted
        transition-colors duration-120 max-radius-2
        ${variantCls}
      `}
    >
      {label}
    </button>
  );
}

// Authorization confirmation panel — used for any backend-issued authorization request.
const AuthorizationPanel = memo(function AuthorizationPanel({
  state,
}: {
  state: AppState;
}) {
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const auth = state.authorization;
  const taskId = state.task?.id;

  const handleApprove = useCallback(async () => {
    if (!taskId || !auth) return;
    setError(null);
    setSubmitting(true);
    try {
      await confirmAuthorization({
        task_id: taskId,
        commitment_hash: auth.commitmentHash,
      });
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setSubmitting(false);
    }
  }, [taskId, auth]);

  const handleDeny = useCallback(async () => {
    if (!taskId) return;
    setError(null);
    setSubmitting(true);
    try {
      await denyAuthorization({ task_id: taskId });
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setSubmitting(false);
    }
  }, [taskId]);

  if (!auth) return null;

  const tierColor =
    auth.tier === "PAYMENT_HANDOFF"
      ? "sem-violation"
      : auth.tier === "REQUIRES_HUMAN"
        ? "sem-caution"
        : "text-secondary";

  return (
    <div className="border-1px line bg-surface-1 p-6">
      <div className="mb-4 flex items-center gap-3">
        <span
          className={`border-1px px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider ${tierColor} border-current`}
        >
          {auth.tier}
        </span>
        <span className="section-header text-primary">AUTHORIZATION REQUIRED</span>
      </div>

      <div className="mb-5 max-w-2xl space-y-3">
        <p className="text-[13px] text-primary leading-relaxed">{auth.summary}</p>
        <div className="flex items-center gap-2 border-t-1px line pt-3">
          <span className="label-uppercase text-muted">COMMITMENT REF</span>
          <span className="data-mono text-[11px] text-secondary">
            {auth.commitmentHash.slice(0, 16)}...
          </span>
        </div>
      </div>

      <div className="flex items-center gap-3">
        <ActionButton
          onClick={() => void handleApprove()}
          disabled={submitting}
          label={submitting ? "SUBMITTING..." : "APPROVE"}
          variant="primary"
        />
        <ActionButton
          onClick={() => void handleDeny()}
          disabled={submitting}
          label={submitting ? "DENYING..." : "DENY"}
          variant="danger"
        />
      </div>

      {error ? (
        <p className="mt-3 text-[11px] data-mono sem-violation">{error}</p>
      ) : null}

      <p className="mt-4 micro-annotation text-muted">
        BACKEND-ISSUED REQUEST · COMMITMENT HASH VERIFIED · TASK PAUSED PENDING DECISION
      </p>
    </div>
  );
});

// Human takeover panel — shows when agent has handed control to the user.
const TakeoverPanel = memo(function TakeoverPanel({ state }: { state: AppState }) {
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const takeover = state.humanTakeover;
  const taskId = state.task?.id;

  const handleResume = useCallback(async () => {
    if (!taskId) return;
    setError(null);
    setSubmitting(true);
    try {
      await endHumanHandoff({ task_id: taskId });
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setSubmitting(false);
    }
  }, [taskId]);

  const handleAbort = useCallback(async () => {
    if (!taskId) return;
    setError(null);
    setSubmitting(true);
    try {
      await abortTask(taskId);
    } catch (err) {
      setError(formatIpcError(err));
    } finally {
      setSubmitting(false);
    }
  }, [taskId]);

  if (!takeover) return null;

  return (
    <div className="border-1px sem-caution bg-surface-1 p-6">
      <div className="mb-4 flex items-center gap-3">
        <span className="bg-sem-caution px-2 py-0.5 text-[10px] font-bold text-bg-0 uppercase tracking-wider">
          HUMAN CONTROL ACTIVE
        </span>
        <span className="section-header text-primary">AUTONOMOUS EXECUTION PAUSED</span>
      </div>

      <div className="mb-5 max-w-2xl space-y-2">
        <p className="text-[13px] text-primary leading-relaxed">
          {takeover.reason ??
            "The agent has transferred browser control to you. Complete your action, then resume."}
        </p>
        <div className="flex items-center gap-2">
          <span className="label-uppercase text-muted">MODE</span>
          <span className="data-mono text-[12px] text-primary">{takeover.mode}</span>
        </div>
        {state.task ? (
          <div className="flex items-center gap-2">
            <span className="label-uppercase text-muted">TASK</span>
            <span className="data-mono text-[12px] text-secondary">
              {state.task.id.slice(0, 12)}... · {state.task.status}
            </span>
          </div>
        ) : null}
      </div>

      <div className="space-y-3">
        <p className="text-[12px] text-secondary">
          When finished with your manual action, click RESUME TASK to return browser
          control to the agent. The agent will take a fresh perception snapshot.
        </p>
        <div className="flex items-center gap-3">
          <ActionButton
            onClick={() => void handleResume()}
            disabled={submitting}
            label={submitting ? "ADVANCING EPOCH..." : "RESUME TASK"}
            variant="primary"
          />
          <ActionButton
            onClick={() => void handleAbort()}
            disabled={submitting}
            label={submitting ? "ABORTING..." : "ABORT TASK"}
            variant="danger"
          />
        </div>
      </div>

      {error ? (
        <p className="mt-3 text-[11px] data-mono sem-violation">{error}</p>
      ) : null}
    </div>
  );
});

function HandoffDeniedNotice({ mode }: { mode: "HANDOFF_DENIED" | "HANDOFF_EXPIRED" }) {
  return (
    <div className="border-1px border-line bg-surface-1 p-6">
      <div className="mb-2 flex items-center gap-3">
        <span className="border-1px sem-violation px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider border-current">
          {mode === "HANDOFF_DENIED" ? "HANDOFF DENIED" : "HANDOFF EXPIRED"}
        </span>
      </div>
      <p className="text-[13px] text-secondary">
        {mode === "HANDOFF_DENIED"
          ? "The human handoff request was denied. The agent will attempt to continue or stop safely."
          : "The human handoff window expired. The agent will attempt safe recovery."}
      </p>
      <p className="mt-3 micro-annotation text-muted">
        REVIEW THE AUDIT LOG FOR FULL DETAILS.
      </p>
    </div>
  );
}

export const InterventionScreen = memo(function InterventionScreen({
  state,
}: {
  state: AppState;
}) {
  const takeover = state.humanTakeover;
  const authorization = state.authorization;
  const hasPendingTakeover =
    takeover?.mode === "HANDOFF_REQUESTED" || takeover?.mode === "HANDOFF_ACTIVE";
  const hasDeniedOrExpired =
    takeover?.mode === "HANDOFF_DENIED" || takeover?.mode === "HANDOFF_EXPIRED";

  const hasAnyIntervention =
    hasPendingTakeover || hasDeniedOrExpired || authorization !== null;

  return (
    <main className="flex min-h-0 flex-1 flex-col">
      <div className="border-b-1px line px-6 py-4">
        <div className="flex items-baseline justify-between">
          <h1 className="section-header text-primary">INTERVENTION QUEUE</h1>
          <span className="micro-annotation text-muted">
            BACKEND-ISSUED CHECKPOINTS ONLY
          </span>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-6">
        {!hasAnyIntervention ? (
          <div className="flex h-full min-h-40 items-center justify-center">
            <div className="text-center">
              <p className="text-[13px] data-mono text-secondary">
                INTERVENTION QUEUE EMPTY
              </p>
              <p className="mt-1 micro-annotation text-muted">
                No pending authorization requests or human checkpoints.
              </p>
            </div>
          </div>
        ) : (
          <div className="max-w-3xl space-y-5">
            {authorization !== null ? (
              <AuthorizationPanel state={state} />
            ) : null}
            {hasPendingTakeover ? <TakeoverPanel state={state} /> : null}
            {hasDeniedOrExpired && takeover ? (
              <HandoffDeniedNotice
                mode={takeover.mode as "HANDOFF_DENIED" | "HANDOFF_EXPIRED"}
              />
            ) : null}
          </div>
        )}
      </div>
    </main>
  );
});
