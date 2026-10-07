import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { StatusSpine } from "./components/status/StatusSpine";
import { ConsoleShell } from "./screens/task-monitor/ConsoleShell";
import { ConfirmationShell } from "./components/confirm/ConfirmationShell";
import { AppState, reduceEvent } from "./state/app";
import "./styles/tailwind.css";

// Browser Agent frontend is a projection of typed IPC events.
// It does not invent security truth (C-17, C-18).
// Security strings render verbatim from the backend (DESIGN.md #20).

export function render(container: HTMLElement, initial: AppState) {
  const root = createRoot(container);

  function app() {
    return (
      <StrictMode>
        <StatusSpine state={initial} />
        <ConsoleShell state={initial} />
        <ConfirmationShell state={initial} />
      </StrictMode>
    );
  }

  root.render(app());
}

export { reduceEvent };
export type { AppState };
