import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
var host = process.env.TAURI_DEV_HOST;
export default defineConfig({
    plugins: [react()],
    clearScreen: false,
    server: {
        host: host !== null && host !== void 0 ? host : '127.0.0.1',
        port: 5173,
        strictPort: true,
        hmr: host
            ? {
                protocol: 'ws',
                host: host,
                port: 5174
            }
            : undefined
    }
});
