import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { memo } from "react";
export const StatusSpine = memo(function StatusSpine({ state }) {
    const egress = state.policy?.verdict === "DENIED" ? "ENFORCED" : "UNKNOWN";
    const modelResidency = state.model?.residency ?? "UNKNOWN";
    const modelVerified = state.model?.verifiedAtLoad ? "VERIFIED" : "NOT_TESTED";
    const highRisk = state.authorization?.tier === "BIOMETRIC_CONFIRM" ? "1/1" : "0/1";
    const audit = state.verification?.outcome === "VERIFIED_SUCCESS"
        ? `CHAIN VERIFIED ${state.verification.timestamp ?? ""}`.trim()
        : "UNKNOWN";
    const taskState = state.task?.status ?? "UNKNOWN";
    const taskStep = state.task?.stepLabel ?? "0 / 0";
    return (_jsx("header", { className: "border-b-1px line-strong border-b bg-surface-1", children: _jsxs("div", { className: "mx-auto flex max-w-7xl items-center gap-6 px-6 py-2 text-secondary data-mono label-uppercase", children: [_jsxs("div", { className: "flex items-center gap-2", children: [_jsx("span", { className: "text-primary text-[11px] font-semibold uppercase tracking-wider-safe", children: "AGENT" }), _jsx("span", { className: "text-muted text-[11px]", children: "BROWSER AGENT" })] }), _jsxs("div", { className: "flex items-center gap-6", children: [_jsxs("div", { className: "flex items-center gap-2 text-[11px]", children: [_jsx("span", { className: "text-muted", children: "EGRESS" }), _jsx("span", { className: "text-primary fixed-width-amount", children: egress })] }), _jsxs("div", { className: "flex items-center gap-2 text-[11px]", children: [_jsx("span", { className: "text-muted", children: "MODEL" }), _jsxs("span", { className: "text-primary fixed-width-amount", children: [state.model?.modelId ?? "UNKNOWN", " \u00B7 ", modelVerified] }), _jsx("span", { className: "text-muted", children: modelResidency })] }), _jsxs("div", { className: "flex items-center gap-2 text-[11px]", children: [_jsx("span", { className: "text-muted", children: "HIGH-RISK" }), _jsx("span", { className: "text-primary fixed-width-amount", children: highRisk })] }), _jsxs("div", { className: "flex items-center gap-2 text-[11px]", children: [_jsx("span", { className: "text-muted", children: "AUDIT" }), _jsx("span", { className: "text-primary fixed-width-amount", children: audit })] }), _jsxs("div", { className: "flex items-center gap-2 text-[11px]", children: [_jsx("span", { className: "text-muted", children: "TASK" }), _jsxs("span", { className: "text-primary fixed-width-amount", children: [taskState, " \u00B7 STEP ", taskStep] })] })] }), _jsx("div", { className: "ml-auto flex items-center gap-3", children: _jsx("span", { className: "text-muted text-[11px]", children: "ABORT" }) })] }) }));
});
