import { readFileSync } from 'node:fs';
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

const pkg = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf8')) as {
  version: string;
};

/**
 * The production shell is served from the fixed first-party origin
 * `browser-agent://ui/`, so the shipped index.html carries a strict Content
 * Security Policy and no runtime CDN or inline script is allowed.
 *
 * The development server is different: Vite injects its client and the React
 * refresh preamble as inline scripts, which the production policy correctly
 * blocks. Rather than weakening the shipped policy, this transform swaps in a
 * development-only policy for `vite dev` and `vite preview` runs. The transform
 * is skipped for real builds, so `vite build` output keeps `script-src 'self'`.
 */
const DEV_CSP = [
  "default-src 'none'",
  "script-src 'self' 'unsafe-inline'",
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data:",
  "font-src 'self'",
  "connect-src 'self' ws:",
  "frame-src 'none'",
  "object-src 'none'",
  "base-uri 'self'",
  "form-action 'none'",
].join('; ');

function devCspTransform(isBuild: boolean) {
  return {
    name: 'browser-agent-dev-csp',
    transformIndexHtml(html: string): string {
      if (isBuild) return html;
      return html.replace(/content="default-src 'none';[^"]*"/, `content="${DEV_CSP}"`);
    },
  };
}

export default defineConfig(({ command }) => ({
  plugins: [react(), devCspTransform(command === 'build')],
  define: {
    // Reported in About/Diagnostics from the real build artifact.
    __BA_APP_VERSION__: JSON.stringify(pkg.version),
  },
  root: '.',
  build: {
    outDir: 'dist',
    sourcemap: false,
    emptyOutDir: true,
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
}));
