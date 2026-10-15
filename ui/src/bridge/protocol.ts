/**
 * Runtime/UI wire contract (Section 20.2).
 *
 * This module is the frontend mirror of `crates/ba-ipc`. It is a *validation*
 * layer, not a convenience wrapper: every inbound line is size-capped, parsed,
 * schema-checked, sequence-checked and structurally validated against a closed
 * per-kind schema before React can see it. Decoded payloads are rebuilt field by
 * field from validated values, so an unexpected property in a frame has no path
 * into application state and cannot grant authority.
 *
 * Nothing here invents state: the only producers of these frames are the Rust
 * core (`ba-core --stdio`) and the managed host adapter.
 */

/** Wire schema version; must equal `ba_domain::SCHEMA_VERSION`. */
export const SCHEMA_VERSION = 1;

/** Hard cap for a single decoded frame; must equal `ba_domain::MAX_IPC_FRAME_BYTES`. */
export const MAX_IPC_FRAME_BYTES = 512 * 1024;

/** Every event kind the runtime may emit, as declared by `EventPayload`. */
export const EVENT_KINDS = [
  'TASK',
  'TASK_STATUS',
  'TASK_ENDED',
  'BROWSER_STATE',
  'TABS',
  'READINESS',
  'APPROVAL_REQUESTED',
  'APPROVAL_CLEARED',
  'HANDOFF_REQUIRED',
  'TASK_HISTORY',
  'NOTES',
  'SKILLS',
  'DOWNLOADS',
  'AUDIT_STATUS',
  'ERROR',
  'COMMAND_REJECTED',
] as const;

export type EventKind = (typeof EVENT_KINDS)[number];

/** Why a frame was refused. A refused frame never reaches application state. */
export type DecodeFailure =
  | 'EMPTY_FRAME'
  | 'OVERSIZED_FRAME'
  | 'INVALID_JSON'
  | 'SCHEMA_MISMATCH'
  | 'UNKNOWN_KIND'
  | 'MALFORMED_PAYLOAD';

export interface DecodeRefusal {
  readonly ok: false;
  readonly failure: DecodeFailure;
  /** Diagnostic detail; rendered only in the redacted diagnostic view. */
  readonly detail: string;
}

/** Budget counter as reported by `TaskBudgetView`. */
export interface BudgetField {
  readonly limit: number;
  readonly used: number;
}

export interface TaskBudgetView {
  readonly action_proposals: BudgetField;
  readonly dispatched_actions: BudgetField;
  readonly navigations: BudgetField;
  readonly input_tokens: BudgetField;
  readonly output_tokens: BudgetField;
  readonly active_ms: BudgetField;
  readonly waiting_ms: number;
  readonly extensions_used: number;
}

/** Authoritative task event as reported by the orchestrator. */
export interface TaskEventView {
  readonly task_id: string;
  readonly revision: number;
  readonly sequence: number;
  readonly state: string;
  readonly wait_reason: string | null;
  readonly action_id: string | null;
  readonly action_class: string | null;
  readonly verification: string | null;
  readonly label: string;
  readonly detail: string;
  readonly at_unix_ms: number;
  readonly monotonic_ms: number;
  readonly elapsed_active_ms: number;
  readonly elapsed_waiting_ms: number;
  readonly budget: TaskBudgetView | null;
}

export interface TaskStatusView {
  readonly task_id: string;
  readonly state: string;
  readonly label: string;
  readonly wait_reason: string | null;
  readonly action_class: string | null;
  readonly verification: string | null;
  readonly budget: TaskBudgetView;
  readonly elapsed_active_ms: number;
  readonly elapsed_waiting_ms: number;
  readonly last_event_at_unix_ms: number;
  readonly task_tabs: readonly string[];
}

export interface BrowserStateView {
  readonly window: string;
  readonly tab: string;
  readonly origin: string;
  readonly context: string;
  readonly private: boolean;
  readonly address: string;
  readonly security_label: string;
  readonly title: string;
  readonly loading_state: string;
  readonly can_go_back: boolean;
  readonly can_go_forward: boolean;
}

export interface ApprovalView {
  readonly action_id: string;
  readonly action: string;
  readonly action_label: string;
  readonly origin: string;
  readonly destination: string | null;
  readonly target_label: string | null;
  /** Digest of the exact action the user is shown; echoed back on approval. */
  readonly digest: string;
  /** Observation epoch shown in the request. */
  readonly epoch: string;
  /** Frame shown in the request. */
  readonly frame: string;
  readonly parameters: readonly (readonly [string, string])[];
  readonly why: string;
  readonly rule: string;
  readonly class: string;
  readonly expected_effect: string;
  readonly undo: string | null;
  readonly expires_at_unix_ms: number;
}

export interface HandoffView {
  readonly action: string;
  readonly action_label: string;
  readonly origin: string;
  readonly destination: string | null;
  readonly target_label: string | null;
  readonly parameters: readonly (readonly [string, string])[];
  readonly consequences: string;
  readonly reason: string;
  readonly rule: string;
  readonly class: string;
  readonly user_action_hint: string;
}

export interface TaskHistoryEntryView {
  readonly task_id: string;
  readonly text: string;
  readonly state: string;
  readonly label: string;
  readonly verification: string | null;
  readonly verification_label: string | null;
  readonly created_at_unix_ms: number;
  readonly ended_at_unix_ms: number;
}

export interface NoteView {
  readonly note: string;
  readonly title: string;
  readonly body: string;
  readonly source_url: string;
  readonly captured_at_unix_ms: number;
}

export interface SkillView {
  readonly skill: string;
  readonly name: string;
  readonly version: string;
  readonly provenance: string;
  readonly enabled: boolean;
}

export interface DownloadView {
  readonly file_name: string;
  readonly origin: string;
  readonly provenance: string;
  readonly progress: string;
}

export interface AuditStatusView {
  readonly healthy: boolean;
  readonly label: string;
}

export interface ReadinessView {
  readonly state: string;
  readonly label: string;
  readonly controls: readonly string[];
  readonly cause: string | null;
  readonly progress: string | null;
}

/** A user-first error following the four-part contract ([BA-ERROR-COPY-001]). */
export interface RuntimeErrorView {
  readonly error_id: string;
  readonly attempted: string;
  readonly observed: string;
  readonly known: string;
  readonly next_action: string;
  readonly rule: string | null;
}

export type DecodedPayload =
  | { readonly kind: 'TASK'; readonly event: TaskEventView }
  | { readonly kind: 'TASK_STATUS'; readonly status: TaskStatusView }
  | {
      readonly kind: 'TASK_ENDED';
      readonly task_id: string;
      readonly state: string;
      readonly state_label: string;
      readonly verification: string | null;
      readonly verification_label: string | null;
      readonly detail: string;
    }
  | { readonly kind: 'BROWSER_STATE'; readonly state: BrowserStateView | null }
  | {
      readonly kind: 'TABS';
      readonly tabs: readonly (readonly [string, string])[];
      readonly active: string | null;
    }
  | { readonly kind: 'READINESS'; readonly readiness: ReadinessView }
  | { readonly kind: 'APPROVAL_REQUESTED'; readonly approval: ApprovalView }
  | { readonly kind: 'APPROVAL_CLEARED'; readonly cause: string; readonly detail: string }
  | { readonly kind: 'HANDOFF_REQUIRED'; readonly handoff: HandoffView }
  | { readonly kind: 'TASK_HISTORY'; readonly entries: readonly TaskHistoryEntryView[] }
  | { readonly kind: 'NOTES'; readonly notes: readonly NoteView[] }
  | { readonly kind: 'SKILLS'; readonly skills: readonly SkillView[] }
  | { readonly kind: 'DOWNLOADS'; readonly downloads: readonly DownloadView[] }
  | { readonly kind: 'AUDIT_STATUS'; readonly audit: AuditStatusView }
  | { readonly kind: 'ERROR'; readonly error: RuntimeErrorView }
  | { readonly kind: 'COMMAND_REJECTED'; readonly command: string; readonly reason: string };

export interface RuntimeEvent {
  readonly sequence: number;
  readonly atUnixMs: number;
  readonly monotonicMs: number;
  readonly taskId: string | null;
  readonly revision: number | null;
  readonly payload: DecodedPayload;
}

export type DecodeResult = { readonly ok: true; readonly event: RuntimeEvent } | DecodeRefusal;

function refuse(failure: DecodeFailure, detail: string): DecodeRefusal {
  return { ok: false, failure, detail };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/**
 * Reads one declared field of an untrusted wire object by its wire name.
 *
 * Fields are addressed by wire name rather than as properties because the input
 * is unknown JSON: nothing is asserted about a value until a typed reader below
 * has accepted it.
 */
function field(record: Record<string, unknown>, name: string): unknown {
  return record[name];
}

function str(value: unknown): string | null {
  return typeof value === 'string' ? value : null;
}

function num(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) ? value : null;
}

function bool(value: unknown): boolean | null {
  return typeof value === 'boolean' ? value : null;
}

function optionalStr(value: unknown): string | null | undefined {
  if (value === null || value === undefined) return null;
  return typeof value === 'string' ? value : undefined;
}

function budgetField(value: unknown): BudgetField | null {
  if (!isRecord(value)) return null;
  const limit = num(field(value, 'limit'));
  const used = num(field(value, 'used'));
  if (limit === null || used === null) return null;
  return { limit, used };
}

function budgetView(value: unknown): TaskBudgetView | null {
  if (!isRecord(value)) return null;
  const actionProposals = budgetField(field(value, 'action_proposals'));
  const dispatched = budgetField(field(value, 'dispatched_actions'));
  const navigations = budgetField(field(value, 'navigations'));
  const inputTokens = budgetField(field(value, 'input_tokens'));
  const outputTokens = budgetField(field(value, 'output_tokens'));
  const activeMs = budgetField(field(value, 'active_ms'));
  const waitingMs = num(field(value, 'waiting_ms'));
  const extensions = num(field(value, 'extensions_used'));
  if (
    actionProposals === null ||
    dispatched === null ||
    navigations === null ||
    inputTokens === null ||
    outputTokens === null ||
    activeMs === null ||
    waitingMs === null ||
    extensions === null
  ) {
    return null;
  }
  return {
    action_proposals: actionProposals,
    dispatched_actions: dispatched,
    navigations,
    input_tokens: inputTokens,
    output_tokens: outputTokens,
    active_ms: activeMs,
    waiting_ms: waitingMs,
    extensions_used: extensions,
  };
}

function parameterPairs(value: unknown): readonly (readonly [string, string])[] | null {
  if (!Array.isArray(value)) return null;
  const pairs: (readonly [string, string])[] = [];
  for (const entry of value) {
    if (!Array.isArray(entry) || entry.length !== 2) return null;
    const [key, val] = entry as [unknown, unknown];
    const label = str(key);
    const text = str(val);
    if (label === null || text === null) return null;
    pairs.push([label, text]);
  }
  return pairs;
}

function taskEventView(value: unknown): TaskEventView | null {
  if (!isRecord(value)) return null;
  const taskId = str(field(value, 'task_id'));
  const revision = num(field(value, 'revision'));
  const sequence = num(field(value, 'sequence'));
  const state = str(field(value, 'state'));
  const waitReason = optionalStr(field(value, 'wait_reason'));
  const actionId = optionalStr(field(value, 'action_id'));
  const actionClass = optionalStr(field(value, 'action_class'));
  const verification = optionalStr(field(value, 'verification'));
  const label = str(field(value, 'label'));
  const detail = str(field(value, 'detail'));
  const atUnixMs = num(field(value, 'at_unix_ms'));
  const monotonicMs = num(field(value, 'monotonic_ms'));
  const elapsedActive = num(field(value, 'elapsed_active_ms'));
  const elapsedWaiting = num(field(value, 'elapsed_waiting_ms'));
  const budgetRaw = field(value, 'budget');
  const budget = budgetRaw === null || budgetRaw === undefined ? null : budgetView(budgetRaw);
  if (
    taskId === null ||
    revision === null ||
    sequence === null ||
    state === null ||
    waitReason === undefined ||
    actionId === undefined ||
    actionClass === undefined ||
    verification === undefined ||
    label === null ||
    detail === null ||
    atUnixMs === null ||
    monotonicMs === null ||
    elapsedActive === null ||
    elapsedWaiting === null ||
    (budgetRaw !== null && budgetRaw !== undefined && budget === null)
  ) {
    return null;
  }
  return {
    task_id: taskId,
    revision,
    sequence,
    state,
    wait_reason: waitReason,
    action_id: actionId,
    action_class: actionClass,
    verification,
    label,
    detail,
    at_unix_ms: atUnixMs,
    monotonic_ms: monotonicMs,
    elapsed_active_ms: elapsedActive,
    elapsed_waiting_ms: elapsedWaiting,
    budget,
  };
}

function taskStatusView(value: unknown): TaskStatusView | null {
  if (!isRecord(value)) return null;
  const taskId = str(field(value, 'task_id'));
  const state = str(field(value, 'state'));
  const label = str(field(value, 'label'));
  const waitReason = optionalStr(field(value, 'wait_reason'));
  const actionClass = optionalStr(field(value, 'action_class'));
  const verification = optionalStr(field(value, 'verification'));
  const budget = budgetView(field(value, 'budget'));
  const elapsedActive = num(field(value, 'elapsed_active_ms'));
  const elapsedWaiting = num(field(value, 'elapsed_waiting_ms'));
  const lastEventAt = num(field(value, 'last_event_at_unix_ms'));
  const taskTabsRaw = field(value, 'task_tabs');
  if (
    taskId === null ||
    state === null ||
    label === null ||
    waitReason === undefined ||
    actionClass === undefined ||
    verification === undefined ||
    budget === null ||
    elapsedActive === null ||
    elapsedWaiting === null ||
    lastEventAt === null ||
    !Array.isArray(taskTabsRaw)
  ) {
    return null;
  }
  const taskTabs: string[] = [];
  for (const tab of taskTabsRaw) {
    const id = str(tab);
    if (id === null) return null;
    taskTabs.push(id);
  }
  return {
    task_id: taskId,
    state,
    label,
    wait_reason: waitReason,
    action_class: actionClass,
    verification,
    budget,
    elapsed_active_ms: elapsedActive,
    elapsed_waiting_ms: elapsedWaiting,
    last_event_at_unix_ms: lastEventAt,
    task_tabs: taskTabs,
  };
}

function browserStateView(value: unknown): BrowserStateView | null {
  if (!isRecord(value)) return null;
  const windowId = str(field(value, 'window'));
  const tab = str(field(value, 'tab'));
  const origin = str(field(value, 'origin'));
  const context = str(field(value, 'context'));
  const isPrivate = bool(field(value, 'private'));
  const address = str(field(value, 'address'));
  const securityLabel = str(field(value, 'security_label'));
  const title = str(field(value, 'title'));
  const loadingState = str(field(value, 'loading_state'));
  const canGoBack = bool(field(value, 'can_go_back'));
  const canGoForward = bool(field(value, 'can_go_forward'));
  if (
    windowId === null ||
    tab === null ||
    origin === null ||
    context === null ||
    isPrivate === null ||
    address === null ||
    securityLabel === null ||
    title === null ||
    loadingState === null ||
    canGoBack === null ||
    canGoForward === null
  ) {
    return null;
  }
  return {
    window: windowId,
    tab,
    origin,
    context,
    private: isPrivate,
    address,
    security_label: securityLabel,
    title,
    loading_state: loadingState,
    can_go_back: canGoBack,
    can_go_forward: canGoForward,
  };
}

function approvalView(value: unknown): ApprovalView | null {
  if (!isRecord(value)) return null;
  const actionId = str(field(value, 'action_id'));
  const action = str(field(value, 'action'));
  const actionLabel = str(field(value, 'action_label'));
  const origin = str(field(value, 'origin'));
  const destination = optionalStr(field(value, 'destination'));
  const targetLabel = optionalStr(field(value, 'target_label'));
  const digest = str(field(value, 'digest'));
  const epoch = str(field(value, 'epoch'));
  const frame = str(field(value, 'frame'));
  const parameters = parameterPairs(field(value, 'parameters'));
  const why = str(field(value, 'why'));
  const rule = str(field(value, 'rule'));
  const consequenceClass = str(field(value, 'class'));
  const expectedEffect = str(field(value, 'expected_effect'));
  const undo = optionalStr(field(value, 'undo'));
  const expiresAt = num(field(value, 'expires_at_unix_ms'));
  if (
    actionId === null ||
    action === null ||
    actionLabel === null ||
    origin === null ||
    destination === undefined ||
    targetLabel === undefined ||
    digest === null ||
    epoch === null ||
    frame === null ||
    parameters === null ||
    why === null ||
    rule === null ||
    consequenceClass === null ||
    expectedEffect === null ||
    undo === undefined ||
    expiresAt === null
  ) {
    return null;
  }
  return {
    action_id: actionId,
    action,
    action_label: actionLabel,
    origin,
    destination,
    target_label: targetLabel,
    digest,
    epoch,
    frame,
    parameters,
    why,
    rule,
    class: consequenceClass,
    expected_effect: expectedEffect,
    undo,
    expires_at_unix_ms: expiresAt,
  };
}

function handoffView(value: unknown): HandoffView | null {
  if (!isRecord(value)) return null;
  const action = str(field(value, 'action'));
  const actionLabel = str(field(value, 'action_label'));
  const origin = str(field(value, 'origin'));
  const destination = optionalStr(field(value, 'destination'));
  const targetLabel = optionalStr(field(value, 'target_label'));
  const parameters = parameterPairs(field(value, 'parameters'));
  const consequences = str(field(value, 'consequences'));
  const reason = str(field(value, 'reason'));
  const rule = str(field(value, 'rule'));
  const consequenceClass = str(field(value, 'class'));
  const hint = str(field(value, 'user_action_hint'));
  if (
    action === null ||
    actionLabel === null ||
    origin === null ||
    destination === undefined ||
    targetLabel === undefined ||
    parameters === null ||
    consequences === null ||
    reason === null ||
    rule === null ||
    consequenceClass === null ||
    hint === null
  ) {
    return null;
  }
  return {
    action,
    action_label: actionLabel,
    origin,
    destination,
    target_label: targetLabel,
    parameters,
    consequences,
    reason,
    rule,
    class: consequenceClass,
    user_action_hint: hint,
  };
}

function historyEntryView(value: unknown): TaskHistoryEntryView | null {
  if (!isRecord(value)) return null;
  const taskId = str(field(value, 'task_id'));
  const text = str(field(value, 'text'));
  const state = str(field(value, 'state'));
  const label = str(field(value, 'label'));
  const verification = optionalStr(field(value, 'verification'));
  const verificationLabel = optionalStr(field(value, 'verification_label'));
  const createdAt = num(field(value, 'created_at_unix_ms'));
  const endedAt = num(field(value, 'ended_at_unix_ms'));
  if (
    taskId === null ||
    text === null ||
    state === null ||
    label === null ||
    verification === undefined ||
    verificationLabel === undefined ||
    createdAt === null ||
    endedAt === null
  ) {
    return null;
  }
  return {
    task_id: taskId,
    text,
    state,
    label,
    verification,
    verification_label: verificationLabel,
    created_at_unix_ms: createdAt,
    ended_at_unix_ms: endedAt,
  };
}

function noteView(value: unknown): NoteView | null {
  if (!isRecord(value)) return null;
  const note = str(field(value, 'note'));
  const title = str(field(value, 'title'));
  const body = str(field(value, 'body'));
  const sourceUrl = str(field(value, 'source_url'));
  const capturedAt = num(field(value, 'captured_at_unix_ms'));
  if (
    note === null ||
    title === null ||
    body === null ||
    sourceUrl === null ||
    capturedAt === null
  ) {
    return null;
  }
  return {
    note,
    title,
    body,
    source_url: sourceUrl,
    captured_at_unix_ms: capturedAt,
  };
}

function skillView(value: unknown): SkillView | null {
  if (!isRecord(value)) return null;
  const skill = str(field(value, 'skill'));
  const name = str(field(value, 'name'));
  const version = str(field(value, 'version'));
  const provenance = str(field(value, 'provenance'));
  const enabled = bool(field(value, 'enabled'));
  if (
    skill === null ||
    name === null ||
    version === null ||
    provenance === null ||
    enabled === null
  ) {
    return null;
  }
  return { skill, name, version, provenance, enabled };
}

function downloadView(value: unknown): DownloadView | null {
  if (!isRecord(value)) return null;
  const fileName = str(field(value, 'file_name'));
  const origin = str(field(value, 'origin'));
  const provenance = str(field(value, 'provenance'));
  const progress = str(field(value, 'progress'));
  if (fileName === null || origin === null || provenance === null || progress === null) {
    return null;
  }
  return { file_name: fileName, origin, provenance, progress };
}

function arrayOf<T>(value: unknown, decode: (entry: unknown) => T | null): readonly T[] | null {
  if (!Array.isArray(value)) return null;
  const decoded: T[] = [];
  for (const entry of value) {
    const item = decode(entry);
    if (item === null) return null;
    decoded.push(item);
  }
  return decoded;
}

function decodePayload(value: unknown): DecodedPayload | null {
  if (!isRecord(value)) return null;
  const kind = str(field(value, 'kind'));
  if (kind === null || !(EVENT_KINDS as readonly string[]).includes(kind)) return null;

  switch (kind as EventKind) {
    case 'TASK': {
      const event = taskEventView(field(value, 'event'));
      return event === null ? null : { kind: 'TASK', event };
    }
    case 'TASK_STATUS': {
      const status = taskStatusView(field(value, 'status'));
      return status === null ? null : { kind: 'TASK_STATUS', status };
    }
    case 'TASK_ENDED': {
      const taskId = str(field(value, 'task_id'));
      const state = str(field(value, 'state'));
      const stateLabel = str(field(value, 'state_label'));
      const verification = optionalStr(field(value, 'verification'));
      const verificationLabel = optionalStr(field(value, 'verification_label'));
      const detail = str(field(value, 'detail'));
      if (
        taskId === null ||
        state === null ||
        stateLabel === null ||
        verification === undefined ||
        verificationLabel === undefined ||
        detail === null
      ) {
        return null;
      }
      return {
        kind: 'TASK_ENDED',
        task_id: taskId,
        state,
        state_label: stateLabel,
        verification,
        verification_label: verificationLabel,
        detail,
      };
    }
    case 'BROWSER_STATE': {
      const raw = field(value, 'state');
      if (raw === null) return { kind: 'BROWSER_STATE', state: null };
      const state = browserStateView(raw);
      return state === null ? null : { kind: 'BROWSER_STATE', state };
    }
    case 'TABS': {
      const tabs = parameterPairs(field(value, 'tabs'));
      const active = optionalStr(field(value, 'active'));
      if (tabs === null || active === undefined) return null;
      return { kind: 'TABS', tabs, active };
    }
    case 'READINESS': {
      const state = str(field(value, 'state'));
      const label = str(field(value, 'label'));
      const controlsRaw = field(value, 'controls');
      const cause = optionalStr(field(value, 'cause'));
      const progress = optionalStr(field(value, 'progress'));
      const controls = arrayOf(controlsRaw, (entry) => str(entry));
      if (
        state === null ||
        label === null ||
        controls === null ||
        cause === undefined ||
        progress === undefined
      ) {
        return null;
      }
      return { kind: 'READINESS', readiness: { state, label, controls, cause, progress } };
    }
    case 'APPROVAL_REQUESTED': {
      const approval = approvalView(field(value, 'approval'));
      return approval === null ? null : { kind: 'APPROVAL_REQUESTED', approval };
    }
    case 'APPROVAL_CLEARED': {
      const cause = str(field(value, 'cause'));
      const detail = str(field(value, 'detail'));
      return cause === null || detail === null ? null : { kind: 'APPROVAL_CLEARED', cause, detail };
    }
    case 'HANDOFF_REQUIRED': {
      const handoff = handoffView(field(value, 'handoff'));
      return handoff === null ? null : { kind: 'HANDOFF_REQUIRED', handoff };
    }
    case 'TASK_HISTORY': {
      const entries = arrayOf(field(value, 'entries'), historyEntryView);
      return entries === null ? null : { kind: 'TASK_HISTORY', entries };
    }
    case 'NOTES': {
      const notes = arrayOf(field(value, 'notes'), noteView);
      return notes === null ? null : { kind: 'NOTES', notes };
    }
    case 'SKILLS': {
      const skills = arrayOf(field(value, 'skills'), skillView);
      return skills === null ? null : { kind: 'SKILLS', skills };
    }
    case 'DOWNLOADS': {
      const downloads = arrayOf(field(value, 'downloads'), downloadView);
      return downloads === null ? null : { kind: 'DOWNLOADS', downloads };
    }
    case 'AUDIT_STATUS': {
      const healthy = bool(field(value, 'healthy'));
      const label = str(field(value, 'label'));
      return healthy === null || label === null
        ? null
        : { kind: 'AUDIT_STATUS', audit: { healthy, label } };
    }
    case 'ERROR': {
      const errorId = str(field(value, 'error_id'));
      const attempted = str(field(value, 'attempted'));
      const observed = str(field(value, 'observed'));
      const known = str(field(value, 'known'));
      const nextAction = str(field(value, 'next_action'));
      const rule = optionalStr(field(value, 'rule'));
      if (
        errorId === null ||
        attempted === null ||
        observed === null ||
        known === null ||
        nextAction === null ||
        rule === undefined
      ) {
        return null;
      }
      return {
        kind: 'ERROR',
        error: {
          error_id: errorId,
          attempted,
          observed,
          known,
          next_action: nextAction,
          rule,
        },
      };
    }
    case 'COMMAND_REJECTED': {
      const command = str(field(value, 'command'));
      const reason = str(field(value, 'reason'));
      return command === null || reason === null
        ? null
        : { kind: 'COMMAND_REJECTED', command, reason };
    }
    /* c8 ignore next 2 -- the kind list above is exhaustive. */
    default:
      return null;
  }
}

/**
 * Validates one JSONL frame from the runtime.
 *
 * Sequence comparison is deliberately *not* performed here: a stateless decoder
 * cannot know the last accepted sequence, and the client owns that check.
 */
export function decodeEventLine(line: string): DecodeResult {
  const trimmed = line.trim();
  if (trimmed.length === 0) return refuse('EMPTY_FRAME', 'The runtime sent an empty line.');

  const bytes = new TextEncoder().encode(trimmed).length;
  if (bytes > MAX_IPC_FRAME_BYTES) {
    return refuse(
      'OVERSIZED_FRAME',
      `The runtime frame was ${bytes} bytes, above the ${MAX_IPC_FRAME_BYTES} byte limit.`,
    );
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(trimmed);
  } catch {
    return refuse('INVALID_JSON', 'The runtime frame was not valid JSON.');
  }
  if (!isRecord(parsed)) return refuse('INVALID_JSON', 'The runtime frame was not a JSON object.');

  const schemaVersion = num(field(parsed, 'schema_version'));
  if (schemaVersion === null) {
    return refuse('SCHEMA_MISMATCH', 'The runtime frame carried no readable schema version.');
  }
  if (schemaVersion !== SCHEMA_VERSION) {
    return refuse(
      'SCHEMA_MISMATCH',
      `The runtime frame used schema version ${schemaVersion}; this interface speaks ${SCHEMA_VERSION}.`,
    );
  }

  const sequence = num(field(parsed, 'sequence'));
  const atUnixMs = num(field(parsed, 'at_unix_ms'));
  const monotonicMs = num(field(parsed, 'monotonic_ms'));
  if (sequence === null || atUnixMs === null || monotonicMs === null) {
    return refuse('MALFORMED_PAYLOAD', 'The runtime frame had no readable sequence or timestamp.');
  }

  // The runtime omits `task_id` and `revision` for events that belong to no
  // task, so absence means absent rather than malformed. A present value that is
  // not readable is still refused.
  const taskIdField = field(parsed, 'task_id');
  const revisionField = field(parsed, 'revision');
  const taskId = taskIdField === undefined ? null : optionalStr(taskIdField);
  const revision = revisionField === undefined ? null : num(revisionField);
  if (taskId === undefined || (revisionField !== undefined && revision === null)) {
    return refuse('MALFORMED_PAYLOAD', 'The runtime frame carried an unreadable task identity.');
  }

  const rawPayload = field(parsed, 'payload');
  const rawKind = isRecord(rawPayload) ? str(field(rawPayload, 'kind')) : null;
  if (rawKind !== null && !(EVENT_KINDS as readonly string[]).includes(rawKind)) {
    return refuse('UNKNOWN_KIND', `The runtime sent an unknown event kind \`${rawKind}\`.`);
  }

  const payload = decodePayload(rawPayload);
  if (payload === null) {
    return refuse(
      'MALFORMED_PAYLOAD',
      `The runtime event \`${rawKind ?? 'unknown'}\` did not match its declared schema.`,
    );
  }

  return { ok: true, event: { sequence, atUnixMs, monotonicMs, taskId, revision, payload } };
}

/** One command body, mirroring `ba_ipc::Command`. */
export interface CommandBody {
  readonly domain: 'browser' | 'window' | 'agent' | 'model' | 'data';
  readonly op: string;
  readonly args: Record<string, unknown>;
}

/** A versioned inbound command envelope, mirroring `ba_ipc::CommandEnvelope`. */
export interface CommandEnvelope {
  readonly schema_version: number;
  readonly sequence: number;
  readonly at_unix_ms: number;
  readonly payload: CommandBody;
}

/**
 * Serializes one command as a single JSONL line.
 *
 * The UI never decides what it may do: it names the operation it wants and the
 * Rust core answers with either real state, a policy refusal or a four-part
 * error.
 */
export function encodeCommandLine(
  command: CommandBody,
  sequence: number,
  atUnixMs: number,
): string {
  const envelope: CommandEnvelope = {
    schema_version: SCHEMA_VERSION,
    sequence,
    at_unix_ms: atUnixMs,
    payload: command,
  };
  return `${JSON.stringify(envelope)}\n`;
}
