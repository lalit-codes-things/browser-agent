import { useRuntimeData } from '../../domain/useRuntimeData';
import { NO_HOST_DETAIL, NO_HOST_TITLE, RuntimeAbsence } from './RuntimeAbsence';

/**
 * Downloads (Section 17.1).
 *
 * Every download carries its initiating origin and who requested it: a file the
 * Agent task caused must be distinguishable from one the user asked for. The
 * list is only ever what the host reported, and the percentage is the host's own
 * textual progress, so the screen never animates a guess.
 */
export function DownloadsView() {
  const { snapshot } = useRuntimeData();

  if (!snapshot.attached) {
    return <RuntimeAbsence title={NO_HOST_TITLE} detail={NO_HOST_DETAIL} />;
  }

  const downloads = snapshot.downloads;
  if (downloads === null || downloads.length === 0) {
    return (
      <div className="ba-secondary-absent">
        <p className="ba-status">No downloads have been requested in this session.</p>
      </div>
    );
  }

  return (
    <div className="ba-secondary-body">
      <table className="ba-table">
        <caption className="ba-visually-hidden">Downloads reported by the managed browser</caption>
        <thead>
          <tr>
            <th scope="col">File</th>
            <th scope="col">From</th>
            <th scope="col">Requested by</th>
            <th scope="col">Progress</th>
          </tr>
        </thead>
        <tbody>
          {downloads.map((download) => (
            <tr key={`${download.origin}-${download.file_name}`}>
              <td className="ba-table-primary">{download.file_name}</td>
              <td>{download.origin}</td>
              <td>{download.provenance}</td>
              <td>{download.progress}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
