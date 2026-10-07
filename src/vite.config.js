import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
// Plain Vite config: @tauri-apps/cli owns the dev-server contract
// (devUrl http://127.0.0.1:1420, strictPort) from tauri.conf.json.
export default defineConfig({
    plugins: [react()],
    clearScreen: false,
    server: {
        port: 1420,
        strictPort: true,
        watch: {
            ignored: ["**/target/**", "**/node_modules/**"],
        },
    },
    build: {
        outDir: "dist",
        emptyOutDir: true,
    },
});
