import typescript from '@rollup/plugin-typescript';
import { dts } from 'rollup-plugin-dts';
import copy from 'rollup-plugin-copy';

export default [
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/index.js',
      format: 'es',
    },
    plugins: [
      typescript(),
      copy({
        targets: [
          {
            src: 'src/generated/smelter/compositor_web_bg.wasm',
            dest: 'dist',
            rename: 'smelter.wasm',
          },
          {
            src: 'src/generated/LICENSE',
            dest: 'dist',
            rename: 'LICENSE-smelter-wasm-bundle',
          },
        ],
      }),
    ],
  },
  {
    input: './src/index.ts',
    output: {
      file: 'dist/index.d.ts',
      format: 'es',
    },
    plugins: [dts()],
  },
];
