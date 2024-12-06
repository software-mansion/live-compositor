import { createContext } from 'react';
import type { InstanceContextStore } from './instanceContextStore.js';
import { LiveInstanceContextStore } from './instanceContextStore.js';
import { AudioContext } from './audioOutputContext.js';
import type { TimeContext } from './timeContext.js';
import { LiveTimeContext } from './timeContext.js';

export type CompositorOutputContext = {
  // global store for the entire LiveCompositor instance
  instanceStore: InstanceContextStore;
  // Audio mixer configuration
  audioContext: AudioContext;
  // Time tracking and handling for blocking tasks
  timeContext: TimeContext;
};

export const LiveCompositorContext = createContext<CompositorOutputContext>({
  instanceStore: new LiveInstanceContextStore(),
  audioContext: new AudioContext(() => {}, false),
  timeContext: new LiveTimeContext(),
});
