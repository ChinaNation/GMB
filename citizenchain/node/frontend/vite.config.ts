import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  // 中文注释：白皮书由 citizenchain/scripts/generate-local-docs.mjs 内置进 bundle;
  // 公民宪法改由链上 runtime API 返回，不再复制仓库 docs/ 静态目录。
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
