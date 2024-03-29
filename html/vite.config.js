import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'
// https://vitejs.dev/config/
export default defineConfig({
  esbuild:{
    pure:['console.log'],
    drop:['debugger']
  },
  plugins: [
    vue(),
  ],
  resolve: {
    alias: [
      { find: /^@\//, replacement: `${path.resolve(__dirname, 'src')}/` },
      { find: /^~/, replacement: '' }
    ]
  },
  server: {
    proxy: {
      // 选项写法
      '/api': {
        target: 'http://127.0.0.1:52800',
        changeOrigin: true,
        // rewrite: (path) => path.replace(/^\/api/, '')
      },
    }
  },
  css:{
    preprocessorOptions:{
      less:{
        javascriptEnabled: true
      }
    }
  }
})
