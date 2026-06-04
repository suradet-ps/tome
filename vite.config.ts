import { resolve } from 'node:path'
import { copyFile, mkdir } from 'node:fs/promises'
import { defineConfig, type Plugin } from 'vite'
import vue from '@vitejs/plugin-vue'

function spaFallback404(): Plugin {
  return {
    name: 'spa-fallback-404',
    apply: 'build',
    async closeBundle() {
      const distDir = resolve(__dirname, 'dist')
      await mkdir(distDir, { recursive: true })
      await copyFile(resolve(distDir, 'index.html'), resolve(distDir, '404.html'))
    },
  }
}

export default defineConfig({
  plugins: [vue(), spaFallback404()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('node_modules/lucide-vue-next')) return 'icons'
          if (id.includes('node_modules/@supabase/supabase-js')) return 'supabase'
          if (id.includes('node_modules/marked') || id.includes('node_modules/highlight.js')) return 'markdown'
          if (id.includes('node_modules/vue') || id.includes('node_modules/pinia') || id.includes('node_modules/vue-router')) {
            return 'vue'
          }
          return undefined
        },
      },
    },
  },
})
