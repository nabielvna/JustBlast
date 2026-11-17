import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig } from 'vite'

export default defineConfig({
  root: 'ui',
  plugins: [svelte()],
  clearScreen: false,
  server: {
    host: '127.0.0.1',
    port: 1420,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/**', '**/crates/**', '**/target/**', '**/.tools/**'] },
  },
  build: { outDir: '../dist', emptyOutDir: true, target: 'es2022' },
})
