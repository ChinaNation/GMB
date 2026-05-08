import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const host = process.env.TAURI_DEV_HOST;
const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [react()],
  // 中文注释：桌面端白皮书/公民宪法 tab 直接加载仓库根 docs/ 下的静态 HTML，
  // 构建时由 Vite 复制进 frontend/dist，避免依赖私有仓库不可访问的 GitHub Pages。
  publicDir: path.resolve(__dirname, '../../../docs'),
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
