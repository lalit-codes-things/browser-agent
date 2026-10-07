import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolvePath } from "vite-plugin-tauri";

export default defineConfig({
  plugins: [
    react(),
    resolvePath() as any,
  ],
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
