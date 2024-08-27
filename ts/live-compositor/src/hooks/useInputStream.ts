import { createContext, useContext, useSyncExternalStore } from 'react';
import { ContextStore, InputStreamInfo } from './store';

export const LiveCompositorContext = createContext<ContextStore>(new ContextStore());

export function useInputStreams(): InputStreamInfo[] {
  const store = useContext(LiveCompositorContext);
  const ctx = useSyncExternalStore(store.subscribe, store.getSnapshot);
  return ctx.inputs;
}
