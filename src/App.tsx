import { listen } from "@tauri-apps/api/event";
import { useEffect, useReducer } from "react";
import { createRoot } from "react-dom/client";
import { StatusSpine } from "./components/status/StatusSpine";
import { ConsoleShell } from "./screens/task-monitor/ConsoleShell";
import { ConfirmationShell } from "./components/confirm/ConfirmationShell";
import { initialState, reduceEvent, type AppEvent, type AppState } from "./state/app";
import "./styles/tailwind.css";

function App() {
  const [state, dispatch] = useReducer(reduceEvent, initialState);

  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;
    void listen<AppEvent>("app-event", (event) => {
      if (active) dispatch(event.payload);
    }).then((cleanup) => {
      if (active) unlisten = cleanup;
      else cleanup();
    });
    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  return (
    <div className="flex min-h-screen flex-col bg-bg-0 text-ink-0">
      <StatusSpine state={state} />
      <ConsoleShell state={state} />
      <ConfirmationShell state={state} />
    </div>
  );
}

export function render(container: HTMLElement, _initial: AppState = initialState) {
  createRoot(container).render(<App />);
}

export { reduceEvent };
export type { AppState };
