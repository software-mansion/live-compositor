import { createContext } from 'react';
import { InstanceContextStore } from './instanceContextStore.js';
import { OutputContext } from './outputContext.js';

type CompositorOutputContext = {
  // global store for the entire LiveCompositor instance
  instanceStore: InstanceContextStore;
  // state specific to the current output
  outputCtx: OutputContext;
};

export const LiveCompositorContext = createContext<CompositorOutputContext>({
  instanceStore: new InstanceContextStore(),
  outputCtx: new OutputContext(() => {}, false),
});

export { InstanceContextStore, OutputContext };
