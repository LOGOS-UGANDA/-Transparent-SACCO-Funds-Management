import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Content Security Policy for the production build.
// Note: no 'unsafe-eval' is needed — the production bundle is eval-free.
const CSP = [
  "default-src 'self'",
  "script-src 'self'",
  "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
  "font-src 'self' https://fonts.gstatic.com",
  "img-src 'self' data:",
  // RPC endpoints for the Logos blockchain (local + public) and wallet websockets
  "connect-src 'self' http://127.0.0.1:8545 ws://127.0.0.1:8545 https://rpc.logos.co wss://rpc.logos.co",
  "object-src 'none'",
  "base-uri 'self'",
  "frame-ancestors 'self'",
].join('; ');

// Inject the CSP meta tag only into the production build. The dev server
// (which uses eval for HMR) intentionally does NOT get a strict CSP.
function cspPlugin() {
  return {
    name: 'inject-csp',
    apply: 'build',
    transformIndexHtml(html) {
      return html.replace(
        '</title>',
        `</title>\n    <meta http-equiv="Content-Security-Policy" content="${CSP}" />`
      );
    },
  };
}

export default defineConfig({
  plugins: [react(), cspPlugin()],
  // Ensure the lodash-style global lookup short-circuits instead of falling
  // back to Function("return this"), keeping the bundle fully CSP-safe.
  define: {
    global: 'globalThis',
  },
  server: {
    port: 3000,
    open: true,
  },
  preview: {
    port: 3000,
    open: true,
  },
  resolve: {
    alias: {
      '@': '/src',
    },
  },
});
