import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { memo } from "react";
const STATUS_COLOR = {
    PENDING: "text-muted",
    RUNNING: "text-primary",
    VERIFIED: "sem-verified",
    LIKELY_SUCCESS: "text-secondary",
    UNKNOWN: "text-muted",
    FAILED: "sem-violation",
    ABORTED: "sem-violation",
    PARKED: "sem-caution",
};
function statusClass(status) {
    return STATUS_COLOR[status] ?? "text-muted";
}
export const TaskLedger = memo(function TaskLedger({ state }) {
    const rows = state.eventLog;
    return (_jsx("div", { className: "overflow-auto rounded-none border-1px line bg-surface-1", children: _jsxs("table", { className: "w-full text-[12px] data-mono", children: [_jsx("thead", { className: "sticky top-0 z-10 border-b-1px line-strong bg-surface-2", children: _jsxs("tr", { children: [_jsx("th", { className: "px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold", children: "TIME" }), _jsx("th", { className: "px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold", children: "EPOCH" }), _jsx("th", { className: "px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold", children: "TARGET" }), _jsx("th", { className: "px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold", children: "ACTION" }), _jsx("th", { className: "px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold", children: "CLASS" }), _jsx("th", { className: "px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold", children: "TIER" }), _jsx("th", { className: "px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold", children: "POLICY" }), _jsx("th", { className: "px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold", children: "EXEC" }), _jsx("th", { className: "px-3 py-2 text-left text-muted uppercase tracking-wider-safe text-[11px] font-semibold", children: "VERIFY" })] }) }), _jsx("tbody", { children: rows.length === 0 ? (_jsx("tr", { children: _jsx("td", { className: "px-3 py-6 text-center text-muted text-[12px]", children: "No events yet." }) })) : (rows.map((event, i) => {
                        const policy = event.type === "POLICY_DECISION" ? event.payload : null;
                        const verification = event.type === "VERIFICATION_OUTCOME" ? event.payload : null;
                        const task = state.task;
                        return (_jsxs("tr", { className: "border-b-1px line", children: [_jsx("td", { className: "px-3 py-2 text-muted", children: i + 1 }), _jsx("td", { className: "px-3 py-2", children: task?.stepLabel ?? "—" }), _jsx("td", { className: "px-3 py-2 text-muted", children: policy?.actionClass ?? "—" }), _jsx("td", { className: "px-3 py-2 text-primary data-mono uppercase", children: event.type.replace("_", " ") }), _jsx("td", { className: "px-3 py-2 text-muted", children: policy?.actionClass ?? "—" }), _jsx("td", { className: `px-3 py-2 ${policy ? "text-primary data-mono" : "text-muted"}`, children: policy?.tier ?? "—" }), _jsx("td", { className: `px-3 py-2 ${policy ? "text-primary" : "text-muted"}`, children: policy?.verdict ?? "—" }), _jsx("td", { className: "px-3 py-2 text-muted", children: "PENDING" }), _jsx("td", { className: `px-3 py-2 ${verification ? "text-primary" : "text-muted"}`, children: verification?.outcome ?? "—" })] }, i));
                    })) })] }) }));
});
