import { useContext, useState } from 'react';
import { LiveCompositorContext } from './index.js';
import type { StreamState } from './instanceContextStore.ts';
import type { RegisterMp4Input } from '../../cjs/index.js';

let nextStreamNumber = 1;

/*
 * Generates unique input stream id that can be used in e.g. Mp4 component
 */
export function useInternalStreamId(): string {
  const ctx = useContext(LiveCompositorContext);
  const [streamNumber, _setStreamNumber] = useState(() => {
    const result = nextStreamNumber;
    nextStreamNumber += 1;
    return result;
  });
  return `output-local:${streamNumber}:${ctx.outputId}`;
}

export type InputStreamInfo = {
  inputId: string;
  videoState?: StreamState;
  audioState?: StreamState;
  offsetMs?: number;
  videoDurationMs?: number;
  audioDurationMs?: number;
};

export interface InternalStreamStore {
  registerMp4(input: RegisterMp4Input): Promise<void>;
}
