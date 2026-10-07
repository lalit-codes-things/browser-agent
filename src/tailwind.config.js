/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./*.tsx",
    "./components/**/*.{ts,tsx}",
    "./screens/**/*.{ts,tsx}",
    "./state/**/*.{ts,tsx}",
    "./ipc/**/*.{ts,tsx}",
  ],
  theme: {
    extend: {
      fontFamily: {
        ui: ["IBM Plex Sans", "system-ui", "-apple-system", "Segoe UI", "Roboto", "sans-serif"],
        data: ["IBM Plex Mono", "ui-monospace", "SF Mono", "Menlo", "Consolas", "monospace"],
      },
      minWidth: {
        xs: "1024px",
      },
    },
  },
  plugins: [],
};
