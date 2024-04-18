import fetch from "node-fetch";
import fs from "fs-extra";
import { promisify } from "util";
import { Stream } from "stream";
import { ChildProcess, SpawnOptions, spawn as nodeSpawn } from "child_process";

const pipeline = promisify(Stream.pipeline);
const child_processes: ChildProcess[] = [];

process.on("exit", () => {
  killAllChildren();
});

export function spawn(
  command: string,
  args: string[],
  options: SpawnOptions,
): SpawnPromise {
  console.log(`Spawning: ${command} ${args.join(" ")}`);
  const child = nodeSpawn(command, args, {
    stdio: "inherit",
    env: {
      ...process.env,
      LIVE_COMPOSITOR_LOGGER_FORMAT: "compact"
    },
    ...options,
  });

  const promise = new Promise<void>((resolve, reject) => {
    child.on("exit", (code) => {
      if (code === 0) {
        console.log(`Command "${command} ${args.join(" ")}" completed successfully.`);
        resolve();
      } else {
        const errorMessage = `Command "${command} ${args.join(" ")}" failed with exit code ${code}.`;
        console.error(errorMessage);
        reject(new Error(errorMessage));
      }
    });
  }) as SpawnPromise;
  promise.child = child;
  child_processes.push(child);
  return promise;
}

/**
 *  Kill all children processes that were started using `spawn()` function.
 */
export function killAllChildren() {
  for (const child of child_processes) {
    try {
      child.kill();
    } catch {
      /* ignore */
    }
  }
}

export interface SpawnPromise extends Promise<void> {
  child: ChildProcess;
}

export async function downloadAsync(
  url: string,
  destination: string,
): Promise<void> {
  const response = await fetch(url, { method: "GET" });
  if (response.status >= 400) {
    const err: any = new Error(`Request to ${url} failed. \n${response.body}`);
    err.response = response;
    throw err;
  }
  if (response.body) {
    await pipeline(response.body, fs.createWriteStream(destination));
  } else {
    throw Error(`Response with empty body.`);
  }
}

export async function sleepAsync(timeout_ms: number): Promise<void> {
  console.log(`Sleeping for ${timeout_ms} ms`);
  await new Promise<void>((res) => {
    setTimeout(() => {
      res();
    }, timeout_ms);
  });
}
