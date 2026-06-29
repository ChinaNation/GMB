import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  // 中文注释:registry 后端同源托管 dist,base 用相对路径以适配任意内网挂载路径。
  base: './',
  plugins: [react()],
  server: {
    port: 5179,
    host: 'localhost',
    strictPort: true,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8899',
        changeOrigin: true
      }
    }
  },
  preview: {
    port: 5179,
    host: 'localhost',
    strictPort: true,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8899',
        changeOrigin: true
      }
    }
  }
});
