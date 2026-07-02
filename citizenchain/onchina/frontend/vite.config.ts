import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  // OnChina 后端同源托管 dist,base 用相对路径以适配任意内网挂载路径。
  base: './',
  plugins: [react()],
  server: {
    port: 5179,
    host: 'localhost',
    strictPort: true,
    proxy: {
      '/api': {
        target: 'https://onchina.local:8964',
        changeOrigin: true,
        secure: false
      }
    }
  },
  preview: {
    port: 5179,
    host: 'localhost',
    strictPort: true,
    proxy: {
      '/api': {
        target: 'https://onchina.local:8964',
        changeOrigin: true,
        secure: false
      }
    }
  }
});
