export type InputStreamInfo = {
  inputId: string;
  // TODO: add input stream state
  // state: 'registered' | 'playing' | 'finished';

  // TODO: add input type (maybe other options too)
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
    this.context = {
      ...this.context,
      inputs: [...this.context.inputs, input],
    };
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
