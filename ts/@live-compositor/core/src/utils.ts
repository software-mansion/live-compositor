import type { Logger } from 'pino';

type ThrottleOptions = {
  logger: Logger;
  timeoutMs: number;
};

export class ThrottledFunction {
  private fn: () => Promise<void>;
  private shouldCall: boolean = false;
  private runningPromise?: Promise<void> = undefined;
  private opts: ThrottleOptions;

  constructor(fn: () => Promise<void>, opts: ThrottleOptions) {
    this.opts = opts;
    this.fn = fn;
  }

  public scheduleCall() {
    this.shouldCall = true;
    if (this.runningPromise) {
      return;
    }
    this.runningPromise = this.doCall();
  }

  public async waitForPendingCalls(): Promise<void> {
    while (this.runningPromise) {
      await this.runningPromise;
    }
  }

  public setFn(fn: () => Promise<void>) {
    this.fn = fn;
  }

  private async doCall() {
    while (this.shouldCall) {
      const start = Date.now();
      this.shouldCall = false;

      try {
        await this.fn();
      } catch (error) {
        this.opts.logger.error(error);
      }

      const timeoutLeft = start + this.opts.timeoutMs - Date.now();
      if (timeoutLeft > 0) {
        await sleep(timeoutLeft);
      }
      this.runningPromise = undefined;
    }
  }
}

export async function sleep(timeoutMs: number): Promise<void> {
  await new Promise<void>(res => {
    setTimeout(() => {
      res();
    }, timeoutMs);
  });
}
