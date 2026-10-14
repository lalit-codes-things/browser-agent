import { fireEvent, render, screen, within } from '@testing-library/react';
import { App } from '../App';
import { COMPOSER_OVER_LIMIT_COPY, MAX_TASK_CODE_POINTS } from '../domain/useAgent';

/**
 * Shell-level proof for the current build:
 *
 * - the shell renders at all (the shipped CSP previously blocked its own bundle);
 * - the Agent panel is permanent, not a drawer or toggle (Section 0.2);
 * - only whitelisted app-owned icon controls exist (Section 2.6);
 * - no task, progress or verification result is fabricated while no native
 *   runtime is attached ([BA-READY-001], Section 12.2);
 * - the composer warns instead of truncating above its code-point ceiling
 *   ([BA-COMPOSER-001]).
 */

function renderShell() {
  return render(<App />);
}

describe('application shell', () => {
  it('renders the tab strip, navigation toolbar, browser view and Agent panel together', () => {
    const { container } = renderShell();

    expect(container.querySelector('.ba-shell')).not.toBeNull();
    expect(container.querySelector('.ba-tabstrip')).not.toBeNull();
    expect(container.querySelector('.ba-toolbar')).not.toBeNull();
    expect(container.querySelector('.ba-browser-view')).not.toBeNull();
    expect(container.querySelector('.ba-agent-panel')).not.toBeNull();
    expect(container.querySelector('.ba-workspace')).not.toBeNull();
  });

  it('keeps the Agent panel permanently visible with no toggle or dismiss control', () => {
    const { container } = renderShell();
    const panel = container.querySelector('.ba-agent-panel');
    expect(panel).not.toBeNull();

    // No control inside or around the shell may remove the Agent panel.
    expect(
      screen.queryByRole('button', { name: /hide agent|close agent|dismiss agent|toggle agent/i }),
    ).toBeNull();
    expect(container.querySelectorAll('.ba-agent-panel')).toHaveLength(1);
  });

  it('states readiness as its own field, separate from task lifecycle', () => {
    renderShell();
    expect(screen.getByText('Model not installed')).toBeTruthy();
    // No simulated task exists, so no lifecycle label is rendered.
    expect(screen.queryByText('Task received')).toBeNull();
    expect(screen.queryByText('Task ended')).toBeNull();
  });

  it('reports a missing native host instead of simulating a page', () => {
    renderShell();
    expect(screen.getByText('Native browser host not attached')).toBeTruthy();
    expect(screen.getByText(/no page can be displayed/i)).toBeTruthy();
    expect(screen.queryByText(/Managed page content renders here/i)).toBeNull();
  });

  it('exposes only icon controls that are permitted and usable', () => {
    const { container } = renderShell();
    const iconControls = [...container.querySelectorAll('.ba-toolbar-icon-control')];

    // Exactly the permitted set, each with an accessible name (Section 2.6).
    expect(iconControls.map((control) => control.getAttribute('aria-label')).sort()).toEqual([
      'Menu',
      'Redo',
      'Reload',
      'Undo',
    ]);
    for (const control of iconControls) {
      expect(control.getAttribute('aria-label')).not.toBeNull();
    }

    // Controls whose operation exists are enabled; Undo and Redo are disabled
    // with the reason stated, because no app-owned operation has a tested
    // compensating action yet (Section 2.7).
    expect(screen.getByRole('button', { name: 'Reload' }).hasAttribute('disabled')).toBe(false);
    expect(screen.getByRole('button', { name: 'Menu' }).hasAttribute('disabled')).toBe(false);

    const undo = screen.getByRole('button', { name: 'Undo' });
    const redo = screen.getByRole('button', { name: 'Redo' });
    expect(undo.hasAttribute('disabled')).toBe(true);
    expect(redo.hasAttribute('disabled')).toBe(true);
    expect(undo.getAttribute('title')).toMatch(/compensating/i);
    expect(redo.getAttribute('title')).toMatch(/compensated/i);
  });

  it('keeps Back and Forward as text-labelled navigation controls, not icons', () => {
    const { container } = renderShell();
    const back = screen.getByRole('button', { name: 'Back' });
    const forward = screen.getByRole('button', { name: 'Forward' });

    expect(back.textContent).toBe('Back');
    expect(forward.textContent).toBe('Forward');
    // Navigation history is runtime state; with no host attached both are
    // unavailable rather than pretending a page exists.
    expect(back.hasAttribute('disabled')).toBe(true);
    expect(forward.hasAttribute('disabled')).toBe(true);
    expect(container.querySelector('.ba-toolbar-ripple')).toBeNull();
  });

  it('opens a labelled overflow menu with real diagnostics', () => {
    renderShell();

    fireEvent.click(screen.getByRole('button', { name: 'Menu' }));
    const menu = screen.getByRole('menu', { name: 'Browser commands' });

    const item = within(menu).getByRole('menuitem', { name: 'About Browser-Agent' });
    fireEvent.click(item);

    const dialog = screen.getByRole('dialog', { name: 'About Browser-Agent' });
    expect(within(dialog).getByText(/Browser-Agent \d+\.\d+\.\d+/)).toBeTruthy();
    expect(within(dialog).getByText(/Not attached/i)).toBeTruthy();
  });

  it('does not fabricate a task when a task is submitted without a runtime', async () => {
    const { container } = renderShell();

    const composer = container.querySelector<HTMLTextAreaElement>('#ba-composer')!;
    fireEvent.change(composer, {
      target: { value: 'Open the pricing page' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Submit' }));

    // The draft is kept and no lifecycle state appears.
    expect(composer.value).toBe('Open the pricing page');
    expect(await screen.findByText(/No task was started/i)).toBeTruthy();
    expect(screen.queryByText('Task received')).toBeNull();
    expect(screen.queryByText('Verified success')).toBeNull();
  });

  it('warns instead of truncating above the composer code-point ceiling', () => {
    const { container } = renderShell();
    const composer = container.querySelector<HTMLTextAreaElement>('#ba-composer')!;
    const overLimit = 'a'.repeat(MAX_TASK_CODE_POINTS + 1);

    fireEvent.change(composer, { target: { value: overLimit } });

    expect(composer.value).toHaveLength(overLimit.length);
    expect(screen.getByText(COMPOSER_OVER_LIMIT_COPY)).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Submit' }).hasAttribute('disabled')).toBe(true);
  });

  it('publishes the disclosure of what the agent can see', () => {
    renderShell();
    expect(screen.getByText('What the agent will see')).toBeTruthy();
    expect(screen.getByText(/does not receive vault values/i)).toBeTruthy();
  });
});
