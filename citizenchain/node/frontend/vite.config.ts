import { defineConfig, type Plugin } from 'vite';
import react from '@vitejs/plugin-react';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const host = process.env.TAURI_DEV_HOST;
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const whitepaperSource = path.resolve(__dirname, '../../../memory/00-vision/GMB_WHITEPAPER.md');

function localWhitepaperPlugin(): Plugin {
  return {
    name: 'citizenchain-local-whitepaper',
    configureServer(server) {
      // 中文注释：开发模式下直接从 memory 真源提供白皮书 Markdown，避免私有仓库 raw 地址 404。
      server.middlewares.use('/GMB_WHITEPAPER.md', (_req, res) => {
        res.setHeader('Content-Type', 'text/markdown; charset=utf-8');
        fs.createReadStream(whitepaperSource).pipe(res);
      });
    },
    generateBundle() {
      // 中文注释：正式构建时把 memory 真源白皮书发射到 dist 根目录，供本地 iframe 页面 fetch。
      this.emitFile({
        type: 'asset',
        fileName: 'GMB_WHITEPAPER.md',
        source: fs.readFileSync(whitepaperSource, 'utf8')
      });
    }
  };
}

export default defineConfig({
  plugins: [react(), localWhitepaperPlugin()],
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
