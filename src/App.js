import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { StatusSpine } from "./components/status/StatusSpine";
import { ConsoleShell } from "./screens/task-monitor/ConsoleShell";
import { ConfirmationShell } from "./components/confirm/ConfirmationShell";
import { reduceEvent } from "./state/app";
import "./styles/tailwind.css";
// Browser Agent frontend is a projection of typed IPC events.
// It does not invent security truth (C-17, C-18).
// Security strings render verbatim from the backend (DESIGN.md #20).
export function render(container, initial) {
    const root = createRoot(container);
    function app() {
        return (_jsxs(StrictMode, { children: [_jsx(StatusSpine, { state: initial }), _jsx(ConsoleShell, { state: initial }), _jsx(ConfirmationShell, { state: initial })] }));
    }
    root.render(app());
}
export { reduceEvent };
