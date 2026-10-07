import { render } from "./App";
const root = document.getElementById("root");
if (!root)
    throw new Error("missing #root");
render(root, __BG_initial_state__ ?? { eventLog: [], task: null, model: null, policy: null, verification: null, authorization: null, ui: { panels: {} } });
