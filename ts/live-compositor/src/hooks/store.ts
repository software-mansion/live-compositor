export type InputStreamInfo = {
  inputId: string;
  videoState?: 'ready' | 'playing' | 'finished';
  audioState?: 'ready' | 'playing' | 'finished';
};

type Context = {
  inputs: InputStreamInfo[];
};

export class ContextStore {
  private context: Context = {
    inputs: [],
  };
  private onChangeCallbacks: Set<() => void> = new Set();

  public addInput(input: InputStreamInfo) {
    const inputs = [...this.context.inputs, input];
    this.context = { ...this.context, inputs };
    this.signalUpdate();
  }

  public updateInput(update: Partial<InputStreamInfo>) {
    const inputs = this.context.inputs.map(input =>
      input.inputId === update.inputId ? { ...input, ...update } : input
    );
    this.context = { ...this.context, inputs };
    this.signalUpdate();
  }

  private signalUpdate() {
    for (const cb of this.onChangeCallbacks) {
      cb();
    }
  }

  public getSnapshot = (): Context => {
    return this.context;
  };

  public subscribe = (onStoreChange: () => void): (() => void) => {
    this.onChangeCallbacks.add(onStoreChange);
    return () => {
      this.onChangeCallbacks.delete(onStoreChange);
    };
  };
}
