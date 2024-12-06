import { createElement, useContext } from 'react';
import type * as Api from '../api.js';
import type { SceneComponent } from '../component.js';
import { createCompositorComponent } from '../component.js';
import { useAudioInput } from '../hooks.js';
import { useTimeLimitedComponent } from '../context/childrenLifetimeContext.js';
import { LiveCompositorContext } from '../context/index.js';
import { OfflineTimeContext } from '../internal.js';

export type Mp4Props = {
  children?: undefined;

  /**
   * Id of a component.
   */
  id?: Api.ComponentId;
  /**
   * Id of an input. It identifies a stream registered using a `LiveCompositor.registerInput`.
   */
  inputId: Api.InputId;
  /**
   * Audio volume represented by a number between 0 and 1.
   */
  volume?: number;
  /**
   * Mute audio.
   */
  muted?: boolean;
};

type AudioPropNames = 'muted' | 'volume';

const InnerMp4 = createCompositorComponent<Omit<Mp4Props, AudioPropNames>>(sceneBuilder);

function Mp4(props: Mp4Props) {
  const { muted, volume, ...otherProps } = props;
  useAudioInput(props.inputId, {
    volume: muted ? 0 : (volume ?? 1),
  });
  useMp4InOfflineContext(props.inputId);
  return createElement(InnerMp4, otherProps);
}

function useMp4InOfflineContext(inputId: string) {
  const ctx = useContext(LiveCompositorContext);
  if (!(ctx.timeContext instanceof OfflineTimeContext)) {
    // condition is constant so it's fine to use hook after that
    return;
  }
  const inputs = useInputStreams();
  const input = inputs[inputId];
  useTimeLimitedComponent((input?.offsetMs ?? 0) + (input?.videoDurationMs ?? 0));
  useTimeLimitedComponent((input?.offsetMs ?? 0) + (input?.audioDurationMs ?? 0));
}

function sceneBuilder(props: Mp4Props, _children: SceneComponent[]): Api.Component {
  return {
    type: 'input_stream',
    id: props.id,
    input_id: props.inputId,
  };
}

export default Mp4;
