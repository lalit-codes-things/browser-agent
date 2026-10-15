import { useEffect } from 'react';
import { SCHEMA_VERSION } from '../../bridge/protocol';
import { useRuntimeData } from '../../domain/useRuntimeData';
import { NO_HOST_TITLE, NO_HOST_DETAIL, RuntimeAbsence } from './RuntimeAbsence';

/**
 * Audit and diagnostics (Sections 15.1, 19).
 *
 * The screen asks the runtime to verify the retained audit chain and shows the
 * answer verbatim. It never claims the chain is verified on its own, and it lists
 * refused frames with their reason, which is the only place a raw protocol
 * diagnostic belongs. Secret material is redacted by the runtime before it ever
 * reaches this screen.
 */
export function AuditView() {
  const data = useRuntimeData();
  const { verifyAudit, snapshot } = data;

  useEffect(() => {
    verifyAudit();
  }, [verifyAudit]);

  if (!snapshot.attached) {
    return <RuntimeAbsence title={NO_HOST_TITLE} detail={NO_HOST_DETAIL} />;
  }

  const audit = snapshot.audit;

  return (
    <div className="ba-secondary-body">
      <dl className="ba-about">
        <dt>Audit chain</dt>
        <dd>{audit === null ? 'Not reported yet.' : audit.label}</dd>
        <dt>Reported state</dt>
        <dd>
          {audit === null
            ? 'The runtime has not answered the verification request.'
            : audit.healthy
              ? 'Healthy'
              : 'Not healthy'}
        </dd>
        <dt>Host adapter</dt>
        <dd>{snapshot.attached ? 'Attached' : 'Not attached'}</dd>
        <dt>Interface schema version</dt>
        <dd>{SCHEMA_VERSION}</dd>
      </dl>

      <h3 className="ba-secondary-subheading">Refused frames</h3>
      {snapshot.refusals === 0 ? (
        <p className="ba-status ba-status--muted">
          No frame from the runtime has been refused in this session.
        </p>
      ) : (
        <>
          <p className="ba-status">
            {snapshot.refusals} frame{snapshot.refusals === 1 ? '' : 's'} refused. A refused frame
            cannot change application state.
          </p>
          {snapshot.lastRefusal !== null && (
            <p className="ba-diagnostic">{snapshot.lastRefusal.detail}</p>
          )}
        </>
      )}

      <button type="button" className="ba-btn ba-btn--secondary" onClick={verifyAudit}>
        Verify audit chain again
      </button>
    </div>
  );
}
