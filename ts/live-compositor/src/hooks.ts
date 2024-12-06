import { useContext, useEffect, useState, useSyncExternalStore } from 'react';

import type * as Api from './api.js';
import type { CompositorOutputContext } from './context/index.js';
import { LiveCompositorContext } from './context/index.js';
import type { InputStreamInfo } from './context/instanceContextStore.js';
import type { BlockingTask } from './context/timeContext.js';
import { OfflineTimeContext } from './context/timeContext.js';

export function useInputStreams(): Record<Api.InputId, InputStreamInfo> {
  const ctx = useContext(LiveCompositorContext);
  const instanceCtx = useSyncExternalStore(
    ctx.instanceStore.subscribe,
    ctx.instanceStore.getSnapshot
  );
  return instanceCtx.inputs;
}

export type AudioOptions = {
  volume: number;
};

/**
 * Hook used to control audio configuration. If you already placing InputStream component
 * you can use `muted` and `volume` props instead.
 */
export function useAudioInput(inputId: Api.InputId, audioOptions: AudioOptions) {
  const ctx = useContext(LiveCompositorContext);

  useEffect(() => {
    const options = { ...audioOptions };
    ctx.audioContext.addInputAudioComponent(inputId, options);
    return () => {
      ctx.audioContext.removeInputAudioComponent(inputId, options);
    };
  }, [audioOptions]);
}

/**
 *  Returns current timestamp relative to `LiveCompositor.start()`.
 *
 *  Not recommended for live processing. It triggers re-renders only for specific timestamps
 *  that are registered with `useAfterTimestamp` hook(that includes components like Slide/Show).
 */
export function useCurrentTimestamp(): number {
  const ctx = useContext(LiveCompositorContext);
  const timeContext = ctx.timeContext;
  useSyncExternalStore(timeContext.subscribe, timeContext.getSnapshot);
  // Value from useSyncExternalStore is the same as TimeContext.timestampMs for
  // offline processing, but for live `timestampMs` should be up to date.
  return timeContext.timestampMs();
}

/**
 * Hook that allows you to trigger updates after specific timestamp. Primary useful for
 * offline processing.
 */
export function useAfterTimestamp(timestamp: number): boolean {
  const ctx = useContext(LiveCompositorContext);
  const currentTimestamp = useCurrentTimestamp();

  useEffect(() => {
    if (timestamp === Infinity) {
      return;
    }
    const tsObject = { timestamp };
    ctx.timeContext.addTimestamp(tsObject);
    return () => {
      ctx.timeContext.removeTimestamp(tsObject);
    };
  }, [timestamp]);

  return currentTimestamp >= timestamp;
}

/**
 * Create task that will stop rendering when compositor runs in offline mode.
 *
 * `task.done()` needs to be called when async action is finished, otherwise rendering will block indefinitely.
 */
export function newBlockingTask(ctx: CompositorOutputContext): BlockingTask {
  if (ctx.timeContext instanceof OfflineTimeContext) {
    return ctx.timeContext.newBlockingTask();
  } else {
    return { done: () => null };
  }
}

/**
 *  Run async function and return its result after Promise resolves.
 *
 *  For offline processing it additionally ensures that rendering for that
 *  timestamp  will block until all blocking tasks are done.
 */
export function useBlockingTask<T>(fn: () => Promise<T>): T | undefined {
  const ctx = useContext(LiveCompositorContext);
  const [result, setResult] = useState<T | undefined>(undefined);
  useEffect(() => {
    const task = newBlockingTask(ctx);
    void (async () => {
      try {
        setResult(await fn());
      } finally {
        task.done();
      }
    })();
    return () => {
      task.done();
    };
  }, []);

  return result;
}
