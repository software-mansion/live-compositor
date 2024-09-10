import { defineConfig } from 'vite';
import { viteStaticCopy } from 'vite-plugin-static-copy';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    viteStaticCopy({
      targets: [
        {
          src: 'node_modules/@live-compositor/browser-render/dist/live-compositor.wasm',
          dest: 'assets',
        },
      ],
    }),
  ],
  optimizeDeps: {
    exclude: ['@rollup/browser'],
  },
});
