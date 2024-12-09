import { createContext } from 'react';
import type { InstanceContextStore } from './instanceContextStore.js';
import { LiveInstanceContextStore } from './instanceContextStore.js';
import { AudioContext } from './audioOutputContext.js';
import type { TimeContext } from './timeContext.js';
import { LiveTimeContext } from './timeContext.js';
import type { InternalStreamStore } from './internalStreamStore.js';

/**
 * Represents ID of an input, it can mean either:
 * - Input registered with `registerInput` method.
 * - Input that was registered internally by components like <Mp4 />.
 */
export type InputRef =
  | {
    // Maps to "global:{id}" in HTTP API
    type: 'global';
    id: string;
  }
  | {
    // Maps to "output-local:{id}:{outputId}" in HTTP API
    type: 'output-local';
    outputId: string;
    id: number;
  };

export type CompositorOutputContext = {
  // global store for the entire LiveCompositor instance
  instanceStore: InstanceContextStore;
  // Audio mixer configuration
  audioContext: AudioContext;
  // Time tracking and handling for blocking tasks
  timeContext: TimeContext;
  // Exposes API to register streams from react that
  // are tied to the specific output stream
  internalStreamsStore: InternalStreamStore;

  outputId: string;
};

export const LiveCompositorContext = createContext<CompositorOutputContext>({
  instanceStore: new LiveInstanceContextStore(),
  audioContext: new AudioContext(() => { }, false),
  timeContext: new LiveTimeContext(),
  outputId: '',
});
