import { useContext, useSyncExternalStore } from 'react';

import * as Api from './api';
import { LiveCompositorContext } from './context';
import { InputStreamInfo } from './context/instanceContextStore';

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
  /*
   * If disabled==true other params will be ignored. You can't use hooks in conditional statements,
   * so you can use that field to disable audio control.
   */
  disabled?: boolean;
};

/**
 * Hook used to control audio configuration. If you already placing InputStream component
 * you can use `mute` and `volume` props instead.
 *
 * When you are using this hook set `disableAudioControl` prop on the InputStream component
 * to avoid race condition.
 */
export function useAudioInput(inputId: Api.InputId, audioOptions: AudioOptions) {
  const ctx = useContext(LiveCompositorContext);
  const { disabled, ...otherOptions } = audioOptions;
  if (disabled !== true) {
    ctx.outputCtx.configureInputAudio(inputId, otherOptions);
  }
}
