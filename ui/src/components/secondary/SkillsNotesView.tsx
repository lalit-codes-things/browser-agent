import { useEffect, useState } from 'react';
import { useRuntimeData } from '../../domain/useRuntimeData';
import { NO_HOST_DETAIL, NO_HOST_TITLE, RuntimeAbsence } from './RuntimeAbsence';

/**
 * Reusable skills and local research notes (Section 17).
 *
 * Skills are versioned and reviewed: the version and its provenance are shown
 * with every entry, and disabling one is explicit. Notes are stored locally with
 * the source URL they came from, so a saved excerpt can always be traced back.
 */
export function SkillsNotesView() {
  const data = useRuntimeData();
  const { requestSkills, requestNotes, revokeSkill, createNote, deleteNote, snapshot } = data;

  const [draftTitle, setDraftTitle] = useState('');
  const [draftBody, setDraftBody] = useState('');
  const [draftSource, setDraftSource] = useState('');

  useEffect(() => {
    requestSkills();
    requestNotes();
  }, [requestNotes, requestSkills]);

  if (!snapshot.attached) {
    return <RuntimeAbsence title={NO_HOST_TITLE} detail={NO_HOST_DETAIL} />;
  }

  const skills = snapshot.skills;
  const notes = snapshot.notes;

  return (
    <div className="ba-secondary-body">
      <h3 className="ba-secondary-subheading">Reviewed skills</h3>
      {skills === null ? (
        <p className="ba-status ba-status--muted">
          The runtime has not reported the skill list yet.
        </p>
      ) : skills.length === 0 ? (
        <p className="ba-status ba-status--muted">
          No skill has been saved and reviewed on this computer.
        </p>
      ) : (
        <ul className="ba-plain-list">
          {skills.map((skill) => (
            <li key={skill.skill} className="ba-plain-list__item">
              <span className="ba-table-primary">
                {skill.name} — version {skill.version}
              </span>
              <span className="ba-table-secondary">
                {skill.provenance}. {skill.enabled ? 'Enabled.' : 'Disabled.'}
              </span>
              <button
                type="button"
                className="ba-btn ba-btn--secondary"
                disabled={!skill.enabled}
                onClick={() => revokeSkill(skill.skill)}
              >
                Disable skill
              </button>
            </li>
          ))}
        </ul>
      )}

      <h3 className="ba-secondary-subheading">Local research notes</h3>
      {notes === null ? (
        <p className="ba-status ba-status--muted">The runtime has not reported notes yet.</p>
      ) : notes.length === 0 ? (
        <p className="ba-status ba-status--muted">No note has been saved on this computer.</p>
      ) : (
        <ul className="ba-plain-list">
          {notes.map((note) => (
            <li key={note.note} className="ba-plain-list__item">
              <span className="ba-table-primary">{note.title}</span>
              <span className="ba-table-secondary">
                {note.source_url} — captured{' '}
                {new Date(note.captured_at_unix_ms).toLocaleString(undefined, {
                  dateStyle: 'medium',
                  timeStyle: 'short',
                })}
              </span>
              <p className="ba-note-body">{note.body}</p>
              <button
                type="button"
                className="ba-btn ba-btn--secondary"
                onClick={() => deleteNote(note.note)}
              >
                Delete note
              </button>
            </li>
          ))}
        </ul>
      )}

      <form
        className="ba-note-form"
        onSubmit={(event) => {
          event.preventDefault();
          if (draftTitle.trim().length === 0 || draftBody.trim().length === 0) return;
          createNote(draftTitle.trim(), draftBody.trim(), draftSource.trim());
          setDraftTitle('');
          setDraftBody('');
          setDraftSource('');
        }}
      >
        <h3 className="ba-secondary-subheading">Add a note</h3>
        <label className="ba-field">
          <span className="ba-field__label">Title</span>
          <input
            className="ba-field__input"
            value={draftTitle}
            onChange={(event) => setDraftTitle(event.target.value)}
          />
        </label>
        <label className="ba-field">
          <span className="ba-field__label">Source URL</span>
          <input
            className="ba-field__input"
            value={draftSource}
            onChange={(event) => setDraftSource(event.target.value)}
            placeholder="https://example.com/page"
            spellCheck={false}
          />
        </label>
        <label className="ba-field">
          <span className="ba-field__label">Note</span>
          <textarea
            className="ba-field__input ba-field__input--multiline"
            value={draftBody}
            onChange={(event) => setDraftBody(event.target.value)}
          />
        </label>
        <button
          type="submit"
          className="ba-btn ba-btn--primary"
          disabled={draftTitle.trim().length === 0 || draftBody.trim().length === 0}
        >
          Save note
        </button>
      </form>
    </div>
  );
}
