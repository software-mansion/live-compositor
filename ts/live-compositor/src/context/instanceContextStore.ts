import * as Api from '../api';

export type InputStreamInfo = {
  inputId: string;
  videoState?: 'ready' | 'playing' | 'finished';
  audioState?: 'ready' | 'playing' | 'finished';
};

type InstanceContext = {
  inputs: Record<Api.InputId, InputStreamInfo>;
};

export class InstanceContextStore {
  private context: InstanceContext = {
    inputs: {},
  };
  private onChangeCallbacks: Set<() => void> = new Set();

  public addInput(input: InputStreamInfo) {
    if (this.context.inputs[input.inputId]) {
      console.warn(`Input ${input.inputId} already exists. Overriding old context.`);
    }
    this.context = {
      ...this.context,
      inputs: {
        ...this.context.inputs,
        [input.inputId]: input,
      },
    };
    this.signalUpdate();
  }

  public updateInput(update: Partial<InputStreamInfo> & { inputId: string }) {
    const oldInput = this.context.inputs[update.inputId];
    if (!oldInput) {
      console.warn('Trying to update input that does not exists.');
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
