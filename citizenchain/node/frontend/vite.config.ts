import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  // 中文注释：文档由 scripts/generate-local-docs.mjs 内置进 bundle；
  // 不再把仓库 docs/ 当静态目录复制，避免旧 HTML 与真源 Markdown 并存。
  publicDir: false,
  clearScreen: false,
  server: {
    host: host ?? '127.0.0.1',
    port: 5173,
    strictPort: true,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 5174
        }
      : undefined
  }
});
