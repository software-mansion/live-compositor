import type { Logger } from 'pino';

type ThrottleOptions = {
  logger: Logger;
  timeoutMs: number;
};

export function throttle(fn: () => Promise<void>, opts: ThrottleOptions): () => void {
  let shouldCall: boolean = false;
  let running: boolean = false;

  const start = async () => {
    while (shouldCall) {
      const start = Date.now();
      shouldCall = false;

      try {
        await fn();
      } catch (error) {
        opts.logger.error(error);
      }

      const timeoutLeft = start + opts.timeoutMs - Date.now();
      if (timeoutLeft > 0) {
        await sleep(timeoutLeft);
      }
      running = false;
    }
  };

  return () => {
    shouldCall = true;
    if (running) {
      return;
    }
    running = true;
    void start();
  };
}

export async function sleep(timeoutMs: number): Promise<void> {
  await new Promise<void>(res => {
    setTimeout(() => {
      res();
    }, timeoutMs);
  });
}
