/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  darkMode: 'class', // Minimal admin panel usually dictates dark mode explicitly or toggled. We'll add 'dark' class to html
  theme: {
    extend: {
      colors: {
        background: '#0f172a',    // slate-900 (Pterodactyl-like dark)
        surface: '#1e293b',       // slate-800
        'surface-hover': '#334155', // slate-700
        primary: '#3b82f6',       // blue-500
        'primary-hover': '#2563eb', // blue-600
        danger: '#ef4444',        // red-500
        success: '#10b981',       // emerald-500
        warning: '#f59e0b',       // amber-500
      },
      fontFamily: {
        sans: ['Inter', 'ui-sans-serif', 'system-ui', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'Helvetica Neue', 'Arial', 'sans-serif'],
        mono: ['Fira Code', 'ui-monospace', 'SFMono-Regular', 'Menlo', 'Monaco', 'Consolas', 'Liberation Mono', 'Courier New', 'monospace'],
      }
    },
  },
  plugins: [],
}
