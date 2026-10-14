import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// The CEF host serves this bundle from the browser-agent://ui/ origin. No
// runtime CDN fetches are allowed for first-party code.
export default defineConfig({
  plugins: [react()],
  root: '.',
  build: {
    outDir: 'dist',
    sourcemap: false,
    emptyOutDir: true,
    rollupOptions: {
      output: {
        // Keep named exports stable for the native bridge contract.
        manualChunks: undefined,
      },
    },
  },
  server: {
    port: 5173,
    host: true,
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
  },
});
