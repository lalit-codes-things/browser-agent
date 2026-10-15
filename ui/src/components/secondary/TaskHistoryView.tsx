import { useEffect } from 'react';
import { useRuntimeData } from '../../domain/useRuntimeData';
import { NO_HOST_DETAIL, NO_HOST_TITLE, RuntimeAbsence } from './RuntimeAbsence';

/**
 * Local Agent task history (Section 15.6, [BA-RESUME-001]).
 *
 * Entries come from the runtime's own record, with the masked instruction text
 * and the canonical terminal label. A task whose result was never verified is
 * labelled from the runtime, never as a success.
 */

function clock(ms: number): string {
  return new Date(ms).toLocaleString(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  });
}

export function TaskHistoryView() {
  const data = useRuntimeData();
  const { requestHistory, clearHistory, snapshot } = data;

  useEffect(() => {
    requestHistory();
  }, [requestHistory]);

  if (!snapshot.attached) {
    return <RuntimeAbsence title={NO_HOST_TITLE} detail={NO_HOST_DETAIL} />;
  }

  const entries = snapshot.history;
  if (entries === null) {
    return (
      <RuntimeAbsence
        title="Task history not reported yet"
        detail="The runtime has not answered the history request, so no entries are shown."
      />
    );
  }

  if (entries.length === 0) {
    return (
      <div className="ba-secondary-absent">
        <p className="ba-status">No tasks have been recorded on this computer.</p>
      </div>
    );
  }

  return (
    <div className="ba-secondary-body">
      <table className="ba-table">
        <caption className="ba-visually-hidden">Local task history, newest first</caption>
        <thead>
          <tr>
            <th scope="col">Task</th>
            <th scope="col">Ended</th>
            <th scope="col">Result</th>
          </tr>
        </thead>
        <tbody>
          {entries.map((entry) => (
            <tr key={entry.task_id}>
              <td>
                <span className="ba-table-primary">{entry.text}</span>
                <span className="ba-table-secondary">{entry.task_id}</span>
              </td>
              <td>{clock(entry.ended_at_unix_ms)}</td>
              <td>
                <span className="ba-table-primary">{entry.label}</span>
                <span className="ba-table-secondary">
                  {entry.verification_label ?? 'No outcome recorded'}
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <div className="ba-secondary-actions">
        <button type="button" className="ba-btn ba-btn--secondary" onClick={clearHistory}>
          Clear task history
        </button>
      </div>
    </div>
  );
}
