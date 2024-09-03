import { spawn } from 'child_process';
import * as path from 'path';

function build() {
  const compositorWebPath = path.resolve(__dirname, '../../../../compositor_web');
  const outputPath = path.resolve(__dirname, '../src/generated');
  const args = ['build', '--target', 'web', '--release', '-d', outputPath, compositorWebPath];

  spawn('wasm-pack', args, { stdio: 'inherit' });
}

build();
