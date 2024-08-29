export function throttle(fn: () => Promise<void>, timeoutMs: number): () => void {
  let shouldCall: boolean = false;
  let running: boolean = false;

  const start = async () => {
    while (shouldCall) {
      const start = Date.now();
      shouldCall = false;

      try {
        await fn();
      } catch (error) {
        console.log(error);
      }

      const timeoutLeft = start + timeoutMs - Date.now();
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

export async function sleep(timeout_ms: number): Promise<void> {
  await new Promise<void>(res => {
    setTimeout(() => {
      res();
    }, timeout_ms);
  });
}
