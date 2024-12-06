import type * as Api from '../api.js';

export type StreamState = 'ready' | 'playing' | 'finished';

export type InputStreamInfo = {
  inputId: string;
  videoState?: StreamState;
  audioState?: StreamState;
  offsetMs?: number;
  videoDurationMs?: number;
  audioDurationMs?: number;
};

type UpdateAction =
  | { type: 'update_input'; input: InputStreamInfo }
  | { type: 'add_input'; input: InputStreamInfo }
  | { type: 'remove_input'; inputId: string };

type InstanceContext = {
  inputs: Record<Api.InputId, InputStreamInfo>;
};

export interface InstanceContextStore {
  getSnapshot: () => InstanceContext;
  subscribe: (onStoreChange: () => void) => () => void;
}

export class LiveInstanceContextStore {
  private context: InstanceContext = {
    inputs: {},
  };
  private onChangeCallbacks: Set<() => void> = new Set();
  private eventQueue?: UpdateAction[];

  /**
   * Apply update immediately if there are no `runBlocking` calls in progress.
   * Otherwise wait for `runBlocking call to finish`.
   */
  public dispatchUpdate(update: UpdateAction) {
    if (this.eventQueue) {
      this.eventQueue.push(update);
    } else {
      this.applyUpdate(update);
    }
  }

  /**
   * No dispatch events will be processed while `fn` function executes.
   * Argument passed to the callback should be used instead of `this.dispatchUpdate`
   * to update the store from inside `fn`
   */
  public async runBlocking<T = void>(
    fn: (update: (action: UpdateAction) => void) => Promise<T>
  ): Promise<T> {
    this.eventQueue = [];
    try {
      return await fn(a => this.applyUpdate(a));
    } finally {
      for (const event of this.eventQueue) {
        this.applyUpdate(event);
      }
      this.eventQueue = undefined;
    }
  }

  private applyUpdate(update: UpdateAction) {
    if (update.type === 'add_input') {
      this.addInput(update.input);
    } else if (update.type === 'update_input') {
      this.updateInput(update.input);
    } else if (update.type === 'remove_input') {
      this.removeInput(update.inputId);
    }
  }

  private addInput(input: InputStreamInfo) {
    if (this.context.inputs[input.inputId]) {
      console.warn(`Adding input ${input.inputId}. Input already exists.`);
    }
    this.context = {
      ...this.context,
      inputs: { ...this.context.inputs, [input.inputId]: input },
    };
    this.signalUpdate();
  }

  private updateInput(update: InputStreamInfo) {
    const oldInput = this.context.inputs[update.inputId];
    if (!oldInput) {
      console.warn(`Updating input ${update.inputId}. Input does not exist.`);
      return;
    }
    this.context = {
      ...this.context,
      inputs: {
        ...this.context.inputs,
        [update.inputId]: { ...oldInput, ...update },
      },
    };
    this.signalUpdate();
  }

  private removeInput(inputId: string) {
    const inputs = { ...this.context.inputs };
    delete inputs[inputId];
    this.context = { ...this.context, inputs };
    this.signalUpdate();
  }

  private signalUpdate() {
    for (const cb of this.onChangeCallbacks) {
      cb();
    }
  }

  // callback for useSyncExternalStore
  public getSnapshot = (): InstanceContext => {
    return this.context;
  };

  // callback for useSyncExternalStore
  public subscribe = (onStoreChange: () => void): (() => void) => {
    this.onChangeCallbacks.add(onStoreChange);
    return () => {
      this.onChangeCallbacks.delete(onStoreChange);
    };
  };
}

type OfflineAddInput = {
  inputId: string;
  offsetMs: number;
  videoDurationMs?: number;
  audioDurationMs?: number;
};

export class OfflineInstanceContextStore {
  private context: InstanceContext = {
    inputs: {},
  };
  private inputs: OfflineAddInput[] = [];
  private onChangeCallbacks: Set<() => void> = new Set();

  public addInput(update: OfflineAddInput) {
    this.inputs.push(update);
  }

  // TimeContext should call that function. It will always trigger re-render, but there
  // is no point to optimize it right now.
  public setCurrentTimestamp(timestampMs: number) {
    const inputs = Object.fromEntries(
      this.inputs
        .filter(input => timestampMs >= input.offsetMs)
        .map(input => {
          // TODO: We could add "unknown" state if Mp4 duration is not known
          const inputState = {
            inputId: input.inputId,
            videoState:
              input.offsetMs + (input.videoDurationMs ?? 0) <= timestampMs ? 'finished' : 'playing',
            audioState:
              input.offsetMs + (input.audioDurationMs ?? 0) <= timestampMs ? 'finished' : 'playing',
            videoDurationMs: input.videoDurationMs,
            audioDurationMs: input.audioDurationMs,
            offsetMs: input.offsetMs,
          } as const;
          return [input.inputId, inputState];
        })
    );
    this.context = { ...this.context, inputs };
    this.signalUpdate();
  }

  private signalUpdate() {
    for (const cb of this.onChangeCallbacks) {
      cb();
    }
  }

  // callback for useSyncExternalStore
  public getSnapshot = (): InstanceContext => {
    return this.context;
  };

  // callback for useSyncExternalStore
  public subscribe = (onStoreChange: () => void): (() => void) => {
    this.onChangeCallbacks.add(onStoreChange);
    return () => {
      this.onChangeCallbacks.delete(onStoreChange);
    };
  };
}
