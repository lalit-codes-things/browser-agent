import { memo } from "react";
import type { AppState, ScreenName } from "../../state/app";

interface NavRailProps {
  currentScreen: ScreenName;
  onSelectScreen: (screen: ScreenName) => void;
  state: AppState;
}

const SCREENS: ScreenName[] = [
  "CONSOLE",
  "INTERVENTION",
  "POLICY",
  "SKILLS",
  "AUDIT",
  "VAULT",
  "MODEL",
  "NETWORK",
  "QUARANTINE",
  "PROFILES",
  "SETTINGS",
];

// Navigation rail (DESIGN.md §11): Text only. No icons.
// Preserves exact industrial aesthetic, denseness, and explicit state indicators.
export const NavRail = memo(function NavRail({
  currentScreen,
  onSelectScreen,
  state,
}: NavRailProps) {
  const isParked = state.task?.status === "PARKED";
  const hasIntervention =
    state.humanTakeover?.mode === "HANDOFF_REQUESTED" ||
    state.humanTakeover?.mode === "HANDOFF_ACTIVE";

  return (
    <nav
      aria-label="Operational Navigation"
      className="flex w-44 shrink-0 flex-col border-r-1px line bg-surface-1 py-3"
    >
      <div className="mb-3 px-4">
        <span className="micro-annotation text-muted tracking-wider-safe">OPERATIONS</span>
      </div>

      <div className="flex flex-1 flex-col gap-0.5">
        {SCREENS.map((screen) => {
          const isActive = currentScreen === screen;
          const showParkedBadge = screen === "INTERVENTION" && isParked;
          const showInterventionBadge = screen === "INTERVENTION" && hasIntervention;

          return (
            <button
              key={screen}
              type="button"
              onClick={() => onSelectScreen(screen)}
              className={`
                flex items-center justify-between px-4 py-2 text-left text-[11px] font-semibold tracking-wider-safe uppercase
                transition-colors duration-120
                ${
                  isActive
                    ? "border-l-2 border-primary bg-surface-3 text-primary"
                    : "border-l-2 border-transparent text-secondary hover:bg-surface-2 hover:text-primary"
                }
              `}
            >
              <span>{screen}</span>
              {showParkedBadge ? (
                <span className="border-1px sem-caution bg-bg-3 px-1 py-0.2 data-mono text-[9px]">
                  1 PARKED
                </span>
              ) : showInterventionBadge ? (
                <span className="border-1px sem-caution bg-bg-3 px-1 py-0.2 data-mono text-[9px]">
                  ACTIVE
                </span>
              ) : null}
            </button>
          );
        })}
      </div>

      <div className="border-t-1px line px-4 pt-3">
        <span className="micro-annotation text-muted">LOCAL RUNTIME</span>
      </div>
    </nav>
  );
});
