import { useContext, useEffect, useSyncExternalStore } from 'react';

import * as Api from './api.js';
import { LiveCompositorContext } from './context/index.js';
import { InputStreamInfo } from './context/instanceContextStore.js';

export function useInputStreams(): Record<Api.InputId, InputStreamInfo> {
  const ctx = useContext(LiveCompositorContext);
  const instanceCtx = useSyncExternalStore(
    ctx.instanceStore.subscribe,
    ctx.instanceStore.getSnapshot
  );
  return instanceCtx.inputs;
}

export type AudioOptions = {
  volume: number;
};

/**
 * Hook used to control audio configuration. If you already placing InputStream component
 * you can use `mute` and `volume` props instead.
 */
export function useAudioInput(inputId: Api.InputId, audioOptions: AudioOptions) {
  const ctx = useContext(LiveCompositorContext);

  useEffect(() => {
    const options = { ...audioOptions };
    ctx.outputCtx.addInputAudioComponent(inputId, options);
    return () => {
      ctx.outputCtx.removeInputAudioComponent(inputId, options);
    };
  }, [audioOptions]);
}
