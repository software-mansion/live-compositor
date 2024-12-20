import { createContext } from 'react';
import { AudioContext } from './audioOutputContext.js';
import type { TimeContext } from './timeContext.js';
import { LiveTimeContext } from './timeContext.js';
import { LiveInputStreamStore, type InputStreamStore } from './inputStreamStore.js';
import type { RegisterMp4Input } from '../types/registerInput.js';
import type { RegisterImage } from '../types/registerRenderer.js';

export type CompositorOutputContext = {
  // global store for input stream state
  globalInputStreamStore: InputStreamStore<string>;
  // internal input streams store
  internalInputStreamStore: InputStreamStore<number>;
  // Audio mixer configuration
  audioContext: AudioContext;
  // Time tracking and handling for blocking tasks
  timeContext: TimeContext;

  outputId: string;

  registerMp4Input: (
    inputId: number,
    registerRequest: RegisterMp4Input
  ) => Promise<{ videoDurationMs?: number; audioDurationMs?: number }>;

  unregisterMp4Input: (inputId: number) => Promise<void>;

  registerImage: (imageId: number, registerRequest: RegisterImage) => Promise<void>;

  unregisterImage: (imageId: number) => Promise<void>;
};

export const LiveCompositorContext = createContext<CompositorOutputContext>({
  globalInputStreamStore: new LiveInputStreamStore(),
  internalInputStreamStore: new LiveInputStreamStore(),
  audioContext: new AudioContext(() => {}),
  timeContext: new LiveTimeContext(),
  outputId: '',
  registerMp4Input: async () => ({}),
  unregisterMp4Input: async () => {},
  registerImage: async () => {},
  unregisterImage: async () => {},
});
