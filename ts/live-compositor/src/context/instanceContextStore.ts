import * as Api from '../api';

export type StreamState = 'ready' | 'playing' | 'finished';

export type InputStreamInfo = {
  inputId: string;
  videoState?: StreamState;
  audioState?: StreamState;
};

type InstanceContext = {
  inputs: Record<Api.InputId, InputStreamInfo>;
};

export class InstanceContextStore {
  private context: InstanceContext = {
    inputs: {},
  };
  private onChangeCallbacks: Set<() => void> = new Set();

  public updateInput(update: InputStreamInfo) {
    const oldInput = this.context.inputs[update.inputId];
    this.context = {
      ...this.context,
      inputs: {
        ...this.context.inputs,
        [update.inputId]: updatedInput(update, oldInput),
      },
    };
    this.signalUpdate();
  }

  public removeInput(inputId: string) {
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

const STATE_CHANGES: Record<StreamState, StreamState[]> = {
  ready: ['playing', 'finished'],
  playing: ['finished'],
  finished: [],
};

function updatedInput(update: InputStreamInfo, oldState?: InputStreamInfo): InputStreamInfo {
  return {
    inputId: update.inputId,
    videoState: updateStreamState(update.videoState, oldState?.videoState),
    audioState: updateStreamState(update.audioState, oldState?.audioState),
  };
}

function updateStreamState(
  update: StreamState | undefined,
  oldState: StreamState | undefined
): StreamState | undefined {
  if (!oldState) {
    return update;
  }
  if (!update) {
    return oldState;
  }
  const validStates = STATE_CHANGES[oldState];
  return validStates.includes(update) ? update : oldState;
}
