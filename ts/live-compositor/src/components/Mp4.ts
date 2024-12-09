import { createElement, useContext } from 'react';
import type * as Api from '../api.js';
import type { SceneComponent } from '../component.js';
import { createCompositorComponent } from '../component.js';
import { useAudioInput, useBlockingTask } from '../hooks.js';
import { useTimeLimitedComponent } from '../context/childrenLifetimeContext.js';
import { LiveCompositorContext } from '../context/index.js';
import { OfflineTimeContext } from '../internal.js';
import { InnerInputStream } from './InputStream.js';
import { useInternalStreamId } from '../context/internalStreamStore.js';

export type Mp4Props = {
  children?: undefined;

  /**
   * Id of a component.
   */
  id?: Api.ComponentId;
  /**
   * Audio volume represented by a number between 0 and 1.
   */
  volume?: number;
  /**
   * Mute audio.
   */
  muted?: boolean;

  source: string;
};

function Mp4(props: Mp4Props) {
  const { muted, volume, ...otherProps } = props;
  const ctx = useContext(LiveCompositorContext);
  const inputId = useInternalStreamId();

  const registerResult = useBlockingTask(async () => {
    ctx.internalStreamsStore.registerMp4();
  });
  useAudioInput(inputId, {
    volume: muted ? 0 : (volume ?? 1),
  });

  return createElement(InnerInputStream, { ...otherProps, inputId });
}

function useMp4InOfflineContext(inputId: string) {
  const ctx = useContext(LiveCompositorContext);
  if (!(ctx.timeContext instanceof OfflineTimeContext)) {
    // condition is constant so it's fine to use hook after that
    return;
  }
  const input = inputs[inputId];
  useTimeLimitedComponent((input?.offsetMs ?? 0) + (input?.videoDurationMs ?? 0));
  useTimeLimitedComponent((input?.offsetMs ?? 0) + (input?.audioDurationMs ?? 0));
}

export default Mp4;
