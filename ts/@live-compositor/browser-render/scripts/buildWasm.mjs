import path from 'node:path';
import { spawn as nodeSpawn } from 'node:child_process';

async function build() {
  const dirName = import.meta.dirname;
  const compositorWebPath = path.resolve(dirName, '../../../../compositor_web');
  const outputPath = path.resolve(dirName, '../src/generated');
  const args = ['build', '--target', 'web', '--release', '-d', outputPath, compositorWebPath];

  return await spawn('wasm-pack', args);
}

function spawn(command, args) {
  const child = nodeSpawn(command, args, {
    stdio: 'inherit',
  });

  return new Promise((resolve, reject) => {
    child.on('exit', code => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`Command "${command} ${args.join(' ')}" failed with exit code ${code}.`));
      }
    });
  });
}

build();
