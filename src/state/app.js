// Frontend application state.
//
// This is a projection of typed backend IPC events. It does not invent
// security truth. Where a backend field is absent, we keep it absent or
// express it as an explicit unknown state (C-01 discipline).
export function reduceEvent(state, event) {
    const next = {
        ...state,
        eventLog: state.eventLog.length >= 500 ? [...state.eventLog.slice(-499), event] : [...state.eventLog, event],
    };
    switch (event.type) {
        case "TASK_STATE":
            return {
                ...next,
                task: {
                    id: event.payload.taskId,
                    status: event.payload.status,
                    stepLabel: event.payload.stepLabel,
                },
            };
        case "MODEL_STATE":
            return { ...next, model: { modelId: event.payload.modelId, residency: event.payload.residency, verifiedAtLoad: event.payload.verifiedAtLoad } };
        case "POLICY_DECISION":
            return { ...next, policy: { actionClass: event.payload.actionClass, tier: event.payload.tier, verdict: event.payload.verdict, reason: event.payload.reason } };
        case "VERIFICATION_OUTCOME":
            return { ...next, verification: { outcome: event.payload.outcome, evidenceCount: event.payload.evidenceCount, timestamp: event.payload.timestamp } };
        case "AUTHORIZATION_REQUIRED":
            return { ...next, authorization: { tier: event.payload.tier, summary: event.payload.summary } };
        default:
            return next;
    }
}
export const initialState = {
    eventLog: [],
    task: null,
    model: null,
    policy: null,
    verification: null,
    authorization: null,
    ui: { panels: {} },
};
