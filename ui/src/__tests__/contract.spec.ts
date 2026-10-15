import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { SCHEMA_VERSION, decodeEventLine } from '../bridge/protocol';
import {
  READINESS_COPY,
  VERIFICATION_COPY,
  taskLifecycleLabel,
  type TaskState,
  type VerificationOutcome,
  type WaitReason,
} from '../domain';

/**
 * Contract drift against the real Rust core (Sections 20.2, 20.4).
 *
 * These tests run the shipped `ba-core` binary rather than a copy of its output:
 * one test diffs the canonical copy table the interface renders, and the other
 * pushes real commands through the stdio loop and decodes every emitted frame
 * with the same decoder the interface uses. A field rename or a copy change on
 * either side fails here instead of reaching a window.
 *
 * When the binary has not been built and `cargo` is unavailable the suite skips
 * rather than passing; CI builds the binary before this command runs.
 */

/** Walks up from the working directory to the repository root. */
function findRepoRoot(): string {
  let dir = process.cwd();
  for (let depth = 0; depth < 4; depth += 1) {
    if (existsSync(join(dir, 'crates', 'ba-runtime', 'Cargo.toml'))) return dir;
    dir = resolve(dir, '..');
  }
  return process.cwd();
}

const REPO_ROOT = findRepoRoot();
const BINARY = process.env.BA_CORE_BIN ?? join(REPO_ROOT, 'target', 'debug', 'ba-core');

function ensureBinary(): string | null {
  if (existsSync(BINARY)) return BINARY;
  try {
    execFileSync('cargo', ['build', '-p', 'ba-runtime', '--bin', 'ba-core'], {
      cwd: REPO_ROOT,
      stdio: 'pipe',
    });
  } catch {
    return null;
  }
  return existsSync(BINARY) ? BINARY : null;
}

const binary = ensureBinary();
const suite = binary === null ? describe.skip : describe;

interface CopyTable {
  schema_version: number;
  status_labels: Record<string, string>;
  task_states: Record<string, { label: string; active_work: boolean }>;
  wait_reasons: Record<string, { label: string; state: string; controls: string[] }>;
  verification_outcomes: Record<
    string,
    { label: string; meaning: string; verified_success: boolean }
  >;
  readiness_states: Record<string, { label: string; controls: string[]; accepts_tasks: boolean }>;
  error_copy: Record<
    string,
    {
      error_id: string;
      attempted: string;
      observed: string;
      known: string;
      next_action: string;
    }
  >;
  stuck: { label: string; controls: string[] };
}

function readCopyTable(): CopyTable {
  const output = execFileSync(binary!, ['export-copy'], { encoding: 'utf8' });
  return JSON.parse(output) as CopyTable;
}

suite('canonical copy contract', () => {
  const table = readCopyTable();

  it('speaks the schema version this interface validates', () => {
    expect(table.schema_version).toBe(SCHEMA_VERSION);
  });

  it('labels every task state exactly as the interface does', () => {
    for (const [state, entry] of Object.entries(table.task_states)) {
      if (state === 'WAITING_FOR_USER') {
        // This state has no reason-independent label: it resolves through its
        // typed wait reason, checked below.
        expect(taskLifecycleLabel(state, null)).toContain('no wait reason');
        continue;
      }
      expect(taskLifecycleLabel(state as TaskState, null)).toBe(entry.label);
    }
    // Every reason-dependent wait label too.
    for (const [reason, entry] of Object.entries(table.wait_reasons)) {
      expect(entry.state).toBe('WAITING_FOR_USER');
      expect(taskLifecycleLabel('WAITING_FOR_USER', reason as WaitReason)).toBe(entry.label);
      expect(table.status_labels[`WAITING_FOR_USER:${reason}`]).toBe(entry.label);
    }
  });

  it('labels every readiness state and lists its supported controls', () => {
    for (const [state, entry] of Object.entries(table.readiness_states)) {
      expect(READINESS_COPY[state as keyof typeof READINESS_COPY]).toBe(entry.label);
      expect(Array.isArray(entry.controls)).toBe(true);
    }
    expect(Object.keys(table.readiness_states).sort()).toEqual(Object.keys(READINESS_COPY).sort());
  });

  it('labels every verification outcome and reserves the success treatment', () => {
    for (const [outcome, entry] of Object.entries(table.verification_outcomes)) {
      const local = VERIFICATION_COPY[outcome as VerificationOutcome];
      expect(local.label).toBe(entry.label);
      expect(local.verifiedSuccess).toBe(entry.verified_success);
      expect(entry.meaning.length).toBeGreaterThan(0);
    }
    expect(Object.keys(table.verification_outcomes).sort()).toEqual(
      Object.keys(VERIFICATION_COPY).sort(),
    );
    expect(table.verification_outcomes.VERIFIED_SUCCESS?.verified_success).toBe(true);
  });

  it('carries the four-part worked examples and no generic filler', () => {
    const required = [
      'NAVIGATION_FAILED',
      'MODEL_NOT_READY',
      'RUNNER_CRASHED',
      'POLICY_BLOCKED',
      'APPROVAL_EXPIRED',
      'VERIFICATION_UNKNOWN',
      'DOWNLOAD_REJECTED',
      'AUDIT_WRITE_FAILED',
      'RESOURCE_STOP',
      'BROWSER_HOST_UNAVAILABLE',
    ];
    expect(Object.keys(table.error_copy).sort()).toEqual([...required].sort());

    for (const [name, copy] of Object.entries(table.error_copy)) {
      expect(copy.error_id, name).toMatch(/^BA-ERR-/);
      expect(copy.attempted.length, name).toBeGreaterThan(0);
      expect(copy.observed.length, name).toBeGreaterThan(0);
      expect(copy.known.length, name).toBeGreaterThan(0);
      expect(copy.next_action.length, name).toBeGreaterThan(0);
      const combined = `${copy.attempted} ${copy.observed} ${copy.known} ${copy.next_action}`;
      expect(combined.toLowerCase(), name).not.toContain('something went wrong');
    }
  });
});

suite('stdio wire contract', () => {
  function runCore(lines: string[]): string[] {
    const result = spawnSync(binary!, ['--stdio'], {
      input: `${lines.join('\n')}\n`,
      encoding: 'utf8',
    });
    expect(result.status).toBe(0);
    return result.stdout.split('\n').filter((line) => line.trim().length > 0);
  }

  function command(sequence: number, payload: Record<string, unknown>): string {
    return JSON.stringify({
      schema_version: SCHEMA_VERSION,
      sequence,
      at_unix_ms: 1_700_000_000_000 + sequence,
      payload,
    });
  }

  const frames = runCore([
    command(1, { domain: 'model', command: { op: 'STATUS', args: {} } }),
    command(2, { domain: 'browser', command: { op: 'NAVIGATE', args: { url: 'https://e.com' } } }),
    command(3, { domain: 'data', command: { op: 'LIST_SKILLS', args: {} } }),
    command(4, { domain: 'data', command: { op: 'VERIFY_AUDIT', args: {} } }),
    // A replayed sequence and an unknown operation must both be refused.
    command(4, { domain: 'data', command: { op: 'LIST_NOTES', args: {} } }),
    command(5, { domain: 'agent', command: { op: 'DELETE_EVERYTHING', args: {} } }),
  ]);

  it('emits only frames this interface accepts', () => {
    expect(frames.length).toBeGreaterThan(0);
    for (const frame of frames) {
      const decoded = decodeEventLine(frame);
      expect(decoded.ok, frame).toBe(true);
    }
  });

  it('answers the commands the interface sends with the kinds it renders', () => {
    const kinds = frames.map((frame) => {
      const decoded = decodeEventLine(frame);
      return decoded.ok ? decoded.event.payload.kind : 'REFUSED';
    });
    expect(kinds).toContain('READINESS');
    expect(kinds).toContain('ERROR');
    expect(kinds).toContain('SKILLS');
    expect(kinds).toContain('AUDIT_STATUS');
    expect(kinds).toContain('COMMAND_REJECTED');
  });

  it('keeps the sequence strictly increasing across the stream', () => {
    const sequences = frames.map((frame) => {
      const decoded = decodeEventLine(frame);
      return decoded.ok ? decoded.event.sequence : -1;
    });
    expect(sequences).toEqual([...sequences].sort((a, b) => a - b));
    expect(new Set(sequences).size).toBe(sequences.length);
  });

  it('reports a missing managed host as an unverified error, not a success', () => {
    const navigation = frames
      .map((frame) => decodeEventLine(frame))
      .find(
        (decoded) =>
          decoded.ok &&
          decoded.event.payload.kind === 'ERROR' &&
          decoded.event.payload.error.error_id === 'BA-ERR-BROWSER-001',
      );
    expect(navigation).toBeDefined();
    if (navigation === undefined || !navigation.ok || navigation.event.payload.kind !== 'ERROR') {
      return;
    }
    expect(navigation.event.payload.error.known).toContain('Nothing was simulated');
    expect(navigation.event.payload.error.next_action.length).toBeGreaterThan(0);
  });
});
