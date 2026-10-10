import { memo } from "react";
import type { AgentCursorSnapshot } from "../../state/app";

interface AgentCursorProps {
  cursor: AgentCursorSnapshot | null;
  epoch?: number;
}

// Agent Cursor (MASTER PROMPT §13, DESIGN.md):
// Visual representation of browser actions, bound to CDP execution and state epoch.
// Industrial precision reticle with state annotations; zero decorative fluff.
export const AgentCursor = memo(function AgentCursor({
  cursor,
  epoch,
}: AgentCursorProps) {
  if (!cursor || !cursor.active) {
    return null;
  }

  const x = cursor.x ?? 120;
  const y = cursor.y ?? 240;

  const stateBorder =
    cursor.state === "BLOCKED"
      ? "border-sem-violation text-sem-violation"
      : cursor.state === "CLICKING"
      ? "border-sem-verified text-sem-verified"
      : cursor.state === "HANDING_OFF"
      ? "border-sem-caution text-sem-caution"
      : "border-primary text-primary";

  return (
    <div
      className="pointer-events-none absolute z-40 transition-transform duration-150 ease-out"
      style={{
        transform: `translate3d(${x}px, ${y}px, 0)`,
      }}
      aria-hidden="true"
    >
      {/* Precision Reticle */}
      <div className="relative">
        {/* Outer Crosshair Box */}
        <div
          className={`h-5 w-5 border-1px bg-bg-0/60 ${stateBorder}`}
          style={{ transform: "translate(-50%, -50%)" }}
        >
          {/* Inner Center Dot */}
          <div className="absolute left-1/2 top-1/2 h-1 w-1 -translate-x-1/2 -translate-y-1/2 bg-current" />
        </div>

        {/* Click Ring Indicator */}
        {cursor.state === "CLICKING" ? (
          <div
            className="absolute left-1/2 top-1/2 h-8 w-8 -translate-x-1/2 -translate-y-1/2 border border-sem-verified opacity-80 animate-ping"
          />
        ) : null}

        {/* Technical Coordinate & State Tag */}
        <div
          className="absolute left-4 top-2 flex flex-col whitespace-nowrap bg-bg-1/95 border-1px line px-1.5 py-0.5 text-[9px] data-mono leading-tight shadow-none"
        >
          <span className="font-semibold">{cursor.state}</span>
          <span className="text-muted">
            {cursor.targetFrameId ? `FRAME: ${cursor.targetFrameId}` : `EPOCH: ${epoch ?? "—"}`}
          </span>
          {cursor.reason ? (
            <span className="text-secondary max-w-[160px] truncate">{cursor.reason}</span>
          ) : null}
        </div>
      </div>
    </div>
  );
});
