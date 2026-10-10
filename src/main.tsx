import { render } from "./App";

// Boot diagnostics carried on the window title.
//
// The Tauri window title tracks `document.title`, so the boot state is
// observable externally (even when devtools are unavailable) and fatal
// startup failures never leave an unexplained white window.
function markBoot(state: string) {
  document.title = `Browser Agent — ${state}`;
}

window.addEventListener("error", (e) => {
  markBoot(`ERROR: ${String(e.message).slice(0, 90)}`);
});
window.addEventListener("unhandledrejection", (e) => {
  markBoot(`REJECTION: ${String(e.reason).slice(0, 90)}`);
});

markBoot("BOOTING");

const root = document.getElementById("root");
if (!root) {
  markBoot("FATAL: missing #root");
  throw new Error("missing #root");
}

try {
  render(root);
  markBoot("READY");
} catch (err) {
  markBoot(`FATAL: ${String(err).slice(0, 90)}`);
  throw err;
}
