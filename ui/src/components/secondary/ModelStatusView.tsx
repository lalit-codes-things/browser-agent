import { useEffect } from 'react';
import { useRuntimeData } from '../../domain/useRuntimeData';
import { asReadinessStatus, UNKNOWN_READINESS_COPY } from '../../domain/state';
import { NO_HOST_DETAIL, NO_HOST_TITLE, RuntimeAbsence } from './RuntimeAbsence';

/**
 * Model status (Section 16, [BA-READY-001]).
 *
 * Readiness, its measured cause and its progress are reported by the runtime.
 * The controls rendered here are exactly the controls the runtime says the state
 * supports, so a state that cannot install cannot offer Install.
 *
 * Two canonical controls have no runtime operation in this build yet: removing
 * an installed model and restarting the runner for the next task. They render
 * disabled with the reason stated, following the same rule as the Undo/Redo
 * controls, rather than pretending to act.
 */

/** Manifest offered when the runtime reports no installed model. */
const FROZEN_MANIFEST_ID = 'qwen3-1.7b-q4_k_m';

export function ModelStatusView() {
  const data = useRuntimeData();
  const {
    requestModelStatus,
    installModel,
    cancelModelInstall,
    verifyModel,
    resumeModel,
    snapshot,
  } = data;

  useEffect(() => {
    requestModelStatus();
  }, [requestModelStatus]);

  if (!snapshot.attached) {
    return <RuntimeAbsence title={NO_HOST_TITLE} detail={NO_HOST_DETAIL} />;
  }

  const readiness = snapshot.readiness;
  if (readiness === null) {
    return (
      <RuntimeAbsence
        title="Model state not reported yet"
        detail="The runtime has not answered the readiness request, so no state is shown."
      />
    );
  }

  const status = asReadinessStatus(readiness.state);
  const label = status === null ? UNKNOWN_READINESS_COPY : readiness.label;
  const controls = status === null ? [] : readiness.controls;
  const offers = (control: string): boolean => controls.includes(control);

  return (
    <div className="ba-secondary-body">
      <dl className="ba-about">
        <dt>State</dt>
        <dd>{label}</dd>
        <dt>Measured detail</dt>
        <dd>{readiness.cause ?? 'No measurement was reported for this state.'}</dd>
        <dt>Progress</dt>
        <dd>{readiness.progress ?? 'Not applicable in this state.'}</dd>
        <dt>Install source</dt>
        <dd>{FROZEN_MANIFEST_ID} — frozen, integrity-checked artifact registered in this build.</dd>
      </dl>

      {status === null && (
        <p className="ba-status ba-status--muted">
          This build does not know the reported state code, so no control is offered. Only the
          runtime may change the model state.
        </p>
      )}

      {status !== null && (
        <div className="ba-secondary-actions">
          {offers('Install model') && (
            <button
              type="button"
              className="ba-btn ba-btn--primary"
              onClick={() => installModel(FROZEN_MANIFEST_ID)}
            >
              Install model
            </button>
          )}
          {offers('Cancel') && (
            <button type="button" className="ba-btn ba-btn--secondary" onClick={cancelModelInstall}>
              Cancel installation
            </button>
          )}
          {offers('Retry verification') && (
            <button type="button" className="ba-btn ba-btn--secondary" onClick={verifyModel}>
              Verify again
            </button>
          )}
          {offers('Remove model') && (
            <button
              type="button"
              className="ba-btn ba-btn--secondary"
              disabled
              title="Removing an installed artifact has no runtime operation in this build yet."
            >
              Remove model
            </button>
          )}
          {offers('Resume model') && (
            <button type="button" className="ba-btn ba-btn--secondary" onClick={resumeModel}>
              Resume model
            </button>
          )}
          {offers('Restart model for the next task') && (
            <button
              type="button"
              className="ba-btn ba-btn--secondary"
              disabled
              title="Restarting the local runner has no runtime operation in this build yet."
            >
              Restart model for the next task
            </button>
          )}
        </div>
      )}
    </div>
  );
}
