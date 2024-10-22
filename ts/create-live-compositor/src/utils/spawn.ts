import type { ChildProcess, SpawnOptions } from 'child_process';
import { spawn as nodeSpawn } from 'child_process';

export interface SpawnPromise extends Promise<{ stdout?: string; stderr?: string }> {
  child: ChildProcess;
}

export function spawn(command: string, args: string[], options: SpawnOptions): SpawnPromise {
  const child = nodeSpawn(command, args, {
    stdio: 'inherit',
    ...options,
  });
  let stdout = '';
  let stderr = '';
  const promise = new Promise((res, rej) => {
    child.on('error', err => {
      rej(err);
    });
    child.on('exit', code => {
      if (code === 0) {
        res({ stdout, stderr });
      } else {
        const err = new Error(
          `Command "${command} ${args.join(' ')}" failed with exit code ${code}.`
        );
        (err as any).stdout = stdout;
        (err as any).stderr = stderr;
        rej(err);
      }
    });
    child.stdout?.on('data', chunk => {
      stdout += chunk.toString();
    });
    child.stderr?.on('data', chunk => {
      stderr += chunk.toString();
    });
  }) as SpawnPromise;
  promise.child = child;
  return promise;
}
