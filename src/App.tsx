import { listen } from "@tauri-apps/api/event";
import { lazy, Suspense, useCallback, useEffect, useReducer } from "react";
import { createRoot } from "react-dom/client";
import { StatusSpine } from "./components/status/StatusSpine";
import { NavRail } from "./components/navigation/NavRail";
import { HumanTakeoverInterlock } from "./components/takeover/HumanTakeoverInterlock";
import { ConfirmationShell } from "./components/confirm/ConfirmationShell";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { ConsoleShell } from "./screens/task-monitor/ConsoleShell";
import { getEgressStatus, getModelStatus } from "./ipc/client";
import {
  initialState,
  reduceEvent,
  type AppEvent,
  type AppState,
  type ScreenName,
} from "./state/app";
import "./styles/tailwind.css";

// Screens are lazy-loaded: they are operational surfaces, not always needed.
// On first activation the module loads; subsequent activations are instant.
const InterventionScreen = lazy(() =>
  import("./screens/intervention/InterventionScreen").then((m) => ({
    default: m.InterventionScreen,
  })),
);
const PolicyScreen = lazy(() =>
  import("./screens/policy/PolicyScreen").then((m) => ({
    default: m.PolicyScreen,
  })),
);
const SkillsScreen = lazy(() =>
  import("./screens/skills/SkillsScreen").then((m) => ({
    default: m.SkillsScreen,
  })),
);
const AuditScreen = lazy(() =>
  import("./screens/audit/AuditScreen").then((m) => ({
    default: m.AuditScreen,
  })),
);
const VaultScreen = lazy(() =>
  import("./screens/vault/VaultScreen").then((m) => ({
    default: m.VaultScreen,
  })),
);
const ModelScreen = lazy(() =>
  import("./screens/model/ModelScreen").then((m) => ({
    default: m.ModelScreen,
  })),
);
const NetworkScreen = lazy(() =>
  import("./screens/network/NetworkScreen").then((m) => ({
    default: m.NetworkScreen,
  })),
);
const QuarantineScreen = lazy(() =>
  import("./screens/quarantine/QuarantineScreen").then((m) => ({
    default: m.QuarantineScreen,
  })),
);
const ProfilesScreen = lazy(() =>
  import("./screens/profiles/ProfilesScreen").then((m) => ({
    default: m.ProfilesScreen,
  })),
);
const SettingsScreen = lazy(() =>
  import("./screens/settings/SettingsScreen").then((m) => ({
    default: m.SettingsScreen,
  })),
);
const StoppedScreen = lazy(() =>
  import("./screens/stopped/StoppedScreen").then((m) => ({
    default: m.StoppedScreen,
  })),
);

// Loading fallback for lazy screens: maintains layout without flicker.
function ScreenLoading() {
  return (
    <div className="flex min-h-0 flex-1 items-center justify-center p-8">
      <span className="micro-annotation text-muted">LOADING SCREEN...</span>
    </div>
  );
}

// Route the active screen based on state.ui.currentScreen.
// Screens are rendered in-place; NavRail controls which is visible.
function ActiveScreen({
  screen,
  state,
  dispatch,
}: {
  screen: ScreenName;
  state: AppState;
  dispatch: React.Dispatch<AppEvent>;
}) {
  switch (screen) {
    case "CONSOLE":
      return <ConsoleShell state={state} />;
    case "INTERVENTION":
      return <InterventionScreen state={state} />;
    case "POLICY":
      return <PolicyScreen state={state} />;
    case "SKILLS":
      return <SkillsScreen />;
    case "AUDIT":
      return <AuditScreen />;
    case "VAULT":
      return <VaultScreen />;
    case "MODEL":
      return <ModelScreen state={state} dispatch={dispatch} />;
    case "NETWORK":
      return <NetworkScreen state={state} dispatch={dispatch} />;
    case "QUARANTINE":
      return <QuarantineScreen />;
    case "PROFILES":
      return <ProfilesScreen />;
    case "SETTINGS":
      return <SettingsScreen />;
    case "STOPPED":
      return <StoppedScreen state={state} dispatch={dispatch} />;
    default: {
      // TypeScript exhaustive guard; runtime fallback avoids blank screen.
      const _exhaustive: never = screen;
      void _exhaustive;
      return <ConsoleShell state={state} />;
    }
  }
}

// Auto-route to INTERVENTION when a human takeover or authorization is pending.
// Auto-route to STOPPED when the task terminates abnormally.
// These transitions are driven by authoritative backend state, not UI guesses.
function deriveAutoScreen(state: AppState): ScreenName | null {
  const { task, humanTakeover, authorization, browser } = state;

  // Hard-stop: route to STOPPED surface regardless of current screen.
  if (
    task?.status === "ABORTED" ||
    task?.status === "FAILED" ||
    browser?.state === "CRASHED"
  ) {
    return "STOPPED";
  }

  // Pending human intervention: surface the INTERVENTION screen.
  if (
    humanTakeover?.mode === "HANDOFF_REQUESTED" ||
    humanTakeover?.mode === "HANDOFF_ACTIVE"
  ) {
    return "INTERVENTION";
  }

  // Backend-issued authorization required.
  if (authorization !== null) {
    return "INTERVENTION";
  }

  return null;
}

function App() {
  const [state, dispatch] = useReducer(reduceEvent, initialState);

  // Screen selection: user-driven unless overridden by auto-route.
  const handleSelectScreen = useCallback(
    (screen: ScreenName) => {
      dispatch({ type: "SET_SCREEN", payload: screen });
    },
    [dispatch],
  );

  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;

    // Single multiplexed event channel from backend.
    void listen<AppEvent>("app-event", (event) => {
      if (active) dispatch(event.payload);
    }).then((cleanup) => {
      if (active) unlisten = cleanup;
      else cleanup();
    });

    // Model and egress state: query once on mount; failures leave explicit unknown state.
    void getModelStatus()
      .then((payload) => {
        if (active) dispatch({ type: "MODEL_STATE", payload });
      })
      .catch(() => undefined);
    void getEgressStatus()
      .then((payload) => {
        if (active) dispatch({ type: "EGRESS_STATE", payload });
      })
      .catch(() => undefined);

    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  // Auto-route overrides manual screen selection for safety-critical states.
  const autoScreen = deriveAutoScreen(state);
  const currentScreen = autoScreen ?? state.ui.currentScreen;

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-bg-0 text-primary">
      {/* Status Spine: always visible, shows runtime vitals. */}
      <StatusSpine state={state} />

      {/* Takeover interlock: always visible when a task is active. */}
      <HumanTakeoverInterlock state={state} />

      {/* Authorization confirmation overlay. */}
      <ConfirmationShell state={state} />

      {/* Main layout: NavRail + content area. */}
      <div className="flex min-h-0 flex-1 overflow-hidden">
        {/* Navigation Rail: text-only, DESIGN.md §11. */}
        <NavRail
          currentScreen={currentScreen}
          onSelectScreen={handleSelectScreen}
          state={state}
        />

        {/* Active screen content area. */}
        <div className="min-h-0 flex-1 overflow-auto">
          <Suspense fallback={<ScreenLoading />}>
            <ActiveScreen
              screen={currentScreen}
              state={state}
              dispatch={dispatch}
            />
          </Suspense>
        </div>
      </div>
    </div>
  );
}

export function render(container: HTMLElement, _initial: AppState = initialState) {
  createRoot(container).render(
    <ErrorBoundary>
      <App />
    </ErrorBoundary>,
  );
}

export { reduceEvent };
export type { AppState };
