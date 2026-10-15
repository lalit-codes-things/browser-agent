import {
  type ApprovalView,
  type AuditStatusView,
  type BrowserStateView,
  type CommandBody,
  type DecodeRefusal,
  type DownloadView,
  type HandoffView,
  type NoteView,
  type ReadinessView,
  type RuntimeErrorView,
  type RuntimeEvent,
  type SkillView,
  type TaskEventView,
  type TaskHistoryEntryView,
  type TaskStatusView,
  decodeEventLine,
  encodeCommandLine,
} from './protocol';

/**
 * The runtime client (Section 20.2).
 *
 * The client owns exactly two things: assigning an outbound sequence to each
 * command and rejecting inbound frames that are stale, malformed or of an
 * unknown kind. It holds no authority: it never invents a task state, never
 * advances a lifecycle and never reports a verification result the runtime did
 * not send. A refused frame is recorded for the redacted diagnostic view and is
 * otherwise dropped.
 *
 * `HostChannel` is the transport seam. The managed host adapter provides it
 * over the CEF message-router boundary; no local HTTP listener and no CDP
 * endpoint exists on any path (Section 14.2).
 */

/** The transport seam between the first-party UI and the Rust core. */
export interface HostChannel {
  /** Sends one JSONL frame to the runtime. */
  send(frame: string): void;
  /** Subscribes to JSONL frames from the runtime. */
  subscribe(listener: (frame: string) => void): () => void;
  /** Host identity, shown only in diagnostics. Never a capability. */
  readonly hostLabel: string;
}

/** Maximum task events kept for the activity list. */
export const MAX_ACTIVITY_EVENTS = 50;

/** Everything the runtime has reported, as of the last accepted event. */
export interface RuntimeSnapshot {
  readonly attached: boolean;
  readonly browser: BrowserStateView | null;
  readonly tabs: readonly (readonly [string, string])[];
  readonly activeTab: string | null;
  readonly readiness: ReadinessView | null;
  readonly task: TaskStatusView | null;
  readonly activity: readonly TaskEventView[];
  readonly ended: {
    readonly task_id: string;
    readonly state: string;
    readonly state_label: string;
    readonly verification: string | null;
    readonly verification_label: string | null;
    readonly detail: string;
  } | null;
  readonly approval: ApprovalView | null;
  readonly handoff: HandoffView | null;
  readonly history: readonly TaskHistoryEntryView[] | null;
  readonly notes: readonly NoteView[] | null;
  readonly skills: readonly SkillView[] | null;
  readonly downloads: readonly DownloadView[] | null;
  readonly audit: AuditStatusView | null;
  readonly error: RuntimeErrorView | null;
  /** Last protocol violation, rendered only in the diagnostic view. */
  readonly lastRefusal: DecodeRefusal | null;
  readonly refusals: number;
}

export interface RuntimeClient {
  readonly hostLabel: string;
  /** Subscribes to snapshot changes. Returns an unsubscribe function. */
  subscribe(listener: () => void): () => void;
  /** Current snapshot; a new object identity after every accepted event. */
  snapshot(): RuntimeSnapshot;
  /** Sends one typed command. */
  send(command: CommandBody): void;
}

function emptySnapshot(attached: boolean): RuntimeSnapshot {
  return {
    attached,
    browser: null,
    tabs: [],
    activeTab: null,
    readiness: null,
    task: null,
    activity: [],
    ended: null,
    approval: null,
    handoff: null,
    history: null,
    notes: null,
    skills: null,
    downloads: null,
    audit: null,
    error: null,
    lastRefusal: null,
    refusals: 0,
  };
}

/**
 * Creates a client bound to a host channel.
 *
 * The channel is supplied by the managed host adapter. Nothing here fabricates
 * frames: when no channel exists the shell is handed no client at all rather
 * than a simulating one.
 */
export function createRuntimeClient(channel: HostChannel): RuntimeClient {
  const listeners = new Set<() => void>();
  let state = emptySnapshot(true);
  let outboundSequence = 0;
  let lastInboundSequence: number | null = null;
  const activity: TaskEventView[] = [];

  const notify = () => {
    for (const listener of listeners) listener();
  };

  const apply = (event: RuntimeEvent): void => {
    const payload = event.payload;
    switch (payload.kind) {
      case 'TASK': {
        activity.push(payload.event);
        if (activity.length > MAX_ACTIVITY_EVENTS) activity.shift();
        state = {
          ...state,
          activity: [...activity],
          ended: null,
        };
        return;
      }
      case 'TASK_STATUS':
        state = { ...state, task: payload.status };
        return;
      case 'TASK_ENDED':
        state = {
          ...state,
          ended: {
            task_id: payload.task_id,
            state: payload.state,
            state_label: payload.state_label,
            verification: payload.verification,
            verification_label: payload.verification_label,
            detail: payload.detail,
          },
          task: null,
          approval: null,
          handoff: null,
        };
        return;
      case 'BROWSER_STATE':
        state = { ...state, browser: payload.state };
        return;
      case 'TABS':
        state = { ...state, tabs: payload.tabs, activeTab: payload.active };
        return;
      case 'READINESS':
        state = { ...state, readiness: payload.readiness };
        return;
      case 'APPROVAL_REQUESTED':
        state = { ...state, approval: payload.approval, handoff: null };
        return;
      case 'APPROVAL_CLEARED':
        state = { ...state, approval: null };
        return;
      case 'HANDOFF_REQUIRED':
        state = { ...state, handoff: payload.handoff, approval: null };
        return;
      case 'TASK_HISTORY':
        state = { ...state, history: payload.entries };
        return;
      case 'NOTES':
        state = { ...state, notes: payload.notes };
        return;
      case 'SKILLS':
        state = { ...state, skills: payload.skills };
        return;
      case 'DOWNLOADS':
        state = { ...state, downloads: payload.downloads };
        return;
      case 'AUDIT_STATUS':
        state = { ...state, audit: payload.audit };
        return;
      case 'ERROR':
        state = { ...state, error: payload.error };
        return;
      case 'COMMAND_REJECTED':
        state = {
          ...state,
          lastRefusal: {
            ok: false,
            failure: 'MALFORMED_PAYLOAD',
            detail: `The runtime refused \`${payload.command}\`: ${payload.reason}`,
          },
          refusals: state.refusals + 1,
        };
        return;
      /* c8 ignore next 2 -- the union above is exhaustive. */
      default:
        return;
    }
  };

  channel.subscribe((frame) => {
    const decoded = decodeEventLine(frame);
    if (!decoded.ok) {
      state = {
        ...state,
        lastRefusal: decoded,
        refusals: state.refusals + 1,
      };
      notify();
      return;
    }
    if (lastInboundSequence !== null && decoded.event.sequence <= lastInboundSequence) {
      state = {
        ...state,
        lastRefusal: {
          ok: false,
          failure: 'SCHEMA_MISMATCH',
          detail: `The runtime event sequence ${decoded.event.sequence} is not after ${lastInboundSequence}, so it was dropped.`,
        },
        refusals: state.refusals + 1,
      };
      notify();
      return;
    }
    lastInboundSequence = decoded.event.sequence;
    apply(decoded.event);
    notify();
  });

  return {
    hostLabel: channel.hostLabel,
    subscribe(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    snapshot: () => state,
    send(command) {
      outboundSequence += 1;
      channel.send(encodeCommandLine(command, outboundSequence, Date.now()));
    },
  };
}

/** Typed command builders. Every operation is one the Rust core implements. */
export const commands = {
  navigate: (url: string): CommandBody => ({ domain: 'browser', op: 'NAVIGATE', args: { url } }),
  back: (): CommandBody => ({ domain: 'browser', op: 'BACK', args: {} }),
  forward: (): CommandBody => ({ domain: 'browser', op: 'FORWARD', args: {} }),
  reload: (): CommandBody => ({ domain: 'browser', op: 'RELOAD', args: {} }),
  stop: (): CommandBody => ({ domain: 'browser', op: 'STOP', args: {} }),
  newTab: (url: string | null): CommandBody => ({
    domain: 'browser',
    op: 'NEW_TAB',
    args: { url },
  }),
  closeTab: (tab: string): CommandBody => ({ domain: 'browser', op: 'CLOSE_TAB', args: { tab } }),
  activateTab: (tab: string): CommandBody => ({
    domain: 'browser',
    op: 'ACTIVATE_TAB',
    args: { tab },
  }),
  findInPage: (query: string): CommandBody => ({
    domain: 'browser',
    op: 'FIND_IN_PAGE',
    args: { query },
  }),
  focusAddressField: (): CommandBody => ({
    domain: 'browser',
    op: 'FOCUS_ADDRESS_FIELD',
    args: {},
  }),
  newWindow: (): CommandBody => ({ domain: 'window', op: 'NEW', args: {} }),
  newPrivateWindow: (): CommandBody => ({ domain: 'window', op: 'NEW_PRIVATE', args: {} }),
  closeWindow: (window: string): CommandBody => ({
    domain: 'window',
    op: 'CLOSE',
    args: { window },
  }),
  submitTask: (text: string, window: string, tab: string): CommandBody => ({
    domain: 'agent',
    op: 'SUBMIT_TASK',
    args: { text, window, tab },
  }),
  pause: (): CommandBody => ({ domain: 'agent', op: 'PAUSE', args: {} }),
  resume: (): CommandBody => ({ domain: 'agent', op: 'RESUME', args: {} }),
  cancelTask: (): CommandBody => ({ domain: 'agent', op: 'CANCEL_TASK', args: {} }),
  takeOver: (): CommandBody => ({ domain: 'agent', op: 'TAKE_OVER', args: {} }),
  approveAction: (
    actionId: string,
    digest: string,
    origin: string,
    epoch: string,
    frame: string,
  ): CommandBody => ({
    domain: 'agent',
    op: 'APPROVE_ACTION',
    args: { action_id: actionId, digest, origin, epoch, frame },
  }),
  cancelApproval: (): CommandBody => ({ domain: 'agent', op: 'CANCEL_APPROVAL', args: {} }),
  extendBudget: (): CommandBody => ({ domain: 'agent', op: 'EXTEND_BUDGET', args: {} }),
  finishWithResults: (): CommandBody => ({
    domain: 'agent',
    op: 'FINISH_WITH_RESULTS',
    args: {},
  }),
  modelStatus: (): CommandBody => ({ domain: 'model', op: 'STATUS', args: {} }),
  modelInstall: (manifestId: string): CommandBody => ({
    domain: 'model',
    op: 'INSTALL',
    args: { manifest_id: manifestId },
  }),
  modelCancelInstall: (): CommandBody => ({ domain: 'model', op: 'CANCEL_INSTALL', args: {} }),
  modelVerify: (): CommandBody => ({ domain: 'model', op: 'VERIFY', args: {} }),
  modelResume: (): CommandBody => ({ domain: 'model', op: 'RESUME', args: {} }),
  forgetSite: (origin: string): CommandBody => ({
    domain: 'data',
    op: 'FORGET_SITE',
    args: { origin },
  }),
  clearBrowsingData: (): CommandBody => ({ domain: 'data', op: 'CLEAR_BROWSING_DATA', args: {} }),
  clearTaskHistory: (): CommandBody => ({ domain: 'data', op: 'CLEAR_TASK_HISTORY', args: {} }),
  listNotes: (): CommandBody => ({ domain: 'data', op: 'LIST_NOTES', args: {} }),
  createNote: (title: string, body: string, sourceUrl: string): CommandBody => ({
    domain: 'data',
    op: 'CREATE_NOTE',
    args: { title, body, source_url: sourceUrl },
  }),
  deleteNote: (note: string): CommandBody => ({
    domain: 'data',
    op: 'DELETE_NOTE',
    args: { note },
  }),
  listSkills: (): CommandBody => ({ domain: 'data', op: 'LIST_SKILLS', args: {} }),
  revokeSkill: (skill: string): CommandBody => ({
    domain: 'data',
    op: 'REVOKE_SKILL',
    args: { skill },
  }),
  verifyAudit: (): CommandBody => ({ domain: 'data', op: 'VERIFY_AUDIT', args: {} }),
  lastTaskResult: (): CommandBody => ({ domain: 'data', op: 'LAST_TASK_RESULT', args: {} }),
} as const;
