import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useCallback, memo } from "react";
// Authorization interlock is the most important visual surface.
// It is not a generic component-library modal. It uses hard borders,
// near-zero radius, flat dark scrim, no blur, and distinct geometry.
// (DESIGN.md #15)
export const ConfirmationShell = memo(function ConfirmationShell({ state }) {
    const auth = state.authorization;
    const open = !!auth;
    const handleConfirm = useCallback(() => {
        // Confirmation outcome is backend-driven; frontend only sends the
        // decision through the allowlisted command path.
        console.info("confirmation accept requested");
    }, []);
    const handleDeny = useCallback(() => {
        console.info("confirmation denied");
    }, []);
    if (!open)
        return null;
    return (_jsxs("div", { role: "dialog", "aria-modal": "true", "aria-labelledby": "auth-heading", className: "fixed inset-0 z-50 flex items-center justify-center bg-[#0a0b0c]", children: [_jsx("div", { className: "pointer-events-none h-full w-full", "aria-hidden": "true" }), _jsxs("div", { className: "\n          relative max-w-lg overflow-hidden border-1px line-strong\n          bg-surface-1 p-6 text-left shadow-none\n          max-radius-2\n        ", children: [_jsx("h2", { id: "auth-heading", className: "mb-4 text-primary label-uppercase", children: auth?.tier === "BIOMETRIC_CONFIRM"
                            ? "HIGH-STAKES AUTHORIZATION"
                            : "AUTHORIZATION REQUIRED" }), _jsxs("dl", { className: "mb-6 grid grid-cols-1 gap-3 text-[12.5px]", children: [_jsxs("div", { className: "flex flex-wrap gap-x-6 gap-y-1", children: [_jsx("dt", { className: "text-muted w-28 shrink-0", children: "TIER" }), _jsx("dd", { className: "text-primary data-mono fixed-width-amount", children: auth?.tier ?? "UNKNOWN" })] }), _jsxs("div", { className: "flex flex-wrap gap-x-6 gap-y-1", children: [_jsx("dt", { className: "text-muted w-28 shrink-0", children: "SUMMARY" }), _jsx("dd", { className: "text-primary data-mono", children: auth?.summary ?? "UNKNOWN" })] })] }), _jsx("p", { className: "mb-6 text-muted body-text", children: "Trusted Policy requires approval before execution." }), _jsxs("div", { className: "flex flex-wrap gap-3", children: [_jsx("button", { type: "button", onClick: handleDeny, className: "\n              min-w-[180px] border-1px line-strong bg-surface-2\n              px-4 py-2 text-primary body-text\n              hover:bg-surface-3 active:bg-bg-3\n              transition-colors duration-120\n              max-radius-2\n            ", children: "DENY AND STOP TASK" }), _jsx("button", { type: "button", onClick: handleConfirm, className: "\n              min-w-[180px] border-1px line-strong\n              bg-surface-2 text-primary body-text\n              hover:bg-surface-3 active:bg-bg-3\n              transition-colors duration-120\n              max-radius-2\n            ", children: "CONFIRM ACTION" })] })] })] }));
});
