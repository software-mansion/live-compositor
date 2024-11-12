import type { ChildProcess, SpawnOptions } from 'child_process';
import { spawn as nodeSpawn } from 'child_process';

export interface SpawnPromise extends Promise<void> {
  child: ChildProcess;
}

export function spawn(command: string, args: string[], options: SpawnOptions): SpawnPromise {
  const child = nodeSpawn(command, args, {
    stdio: 'inherit',
    ...options,
  });
  const promise = new Promise((res, rej) => {
    child.on('error', err => {
      rej(err);
    });
    child.on('exit', code => {
      if (code === 0) {
        res();
      } else {
        rej(new Error(`Command "${command} ${args.join(' ')}" failed with exit code ${code}.`));
      }
    });
  }) as SpawnPromise;
  promise.child = child;
  return promise;
}
