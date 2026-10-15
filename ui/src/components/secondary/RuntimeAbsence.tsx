/**
 * Shown by every secondary screen when no managed host has attached, or when the
 * runtime has not answered yet. A screen reports absence instead of rendering
 * sample rows, a spinner or a progress claim it cannot substantiate
 * ([BA-STATE-001], Section 20.1).
 */
export function RuntimeAbsence({ title, detail }: { title: string; detail: string }) {
  return (
    <div className="ba-secondary-absent">
      <p className="ba-status">{title}</p>
      <p className="ba-status ba-status--muted">{detail}</p>
    </div>
  );
}

export const NO_HOST_TITLE = 'Managed browser host not attached';

export const NO_HOST_DETAIL =
  'The application core is not connected, so this screen has no runtime data to show. Nothing is simulated in its place.';
