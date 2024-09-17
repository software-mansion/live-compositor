import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { viteStaticCopy } from 'vite-plugin-static-copy';
import { createRequire } from 'node:module';
import path from 'node:path';

const require = createRequire(import.meta.url);

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    viteStaticCopy({
      targets: [
        {
          src: path.join(path.dirname(require.resolve('@live-compositor/browser-render')), 'live-compositor.wasm'),
          dest: 'assets',
        },
      ],
    }),
  ],
  optimizeDeps: {
    exclude: ['@rollup/browser'],
  },
});
