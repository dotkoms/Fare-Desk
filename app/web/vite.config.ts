import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    // Lets a dev server on any port reach the local data plane without a CORS allowance:
    // run with VITE_API_BASE pointing at this server's own origin.
    // API_PROXY_TARGET points it at another data plane, e.g. a branch build on :8081.
    proxy: {
      '/api': process.env.API_PROXY_TARGET ?? 'http://127.0.0.1:8080',
      '/healthz': process.env.API_PROXY_TARGET ?? 'http://127.0.0.1:8080',
    },
  },
})
