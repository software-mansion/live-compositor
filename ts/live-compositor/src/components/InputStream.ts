import { createElement, useContext } from 'react';
import type * as Api from '../api.js';
import type { SceneComponent } from '../component.js';
import { createCompositorComponent } from '../component.js';
import { useAudioInput, useInputStreams } from '../hooks.js';
import { useTimeLimitedComponent } from '../context/childrenLifetimeContext.js';
import { LiveCompositorContext } from '../context/index.js';
import { OfflineTimeContext } from '../internal.js';

export type InputStreamProps = {
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

export const InnerInputStream =
  createCompositorComponent<Omit<InputStreamProps, AudioPropNames>>(sceneBuilder);

function InputStream(props: InputStreamProps) {
  const { muted, volume, inputId, ...otherProps } = props;
  useAudioInput(inputId, {
    volume: muted ? 0 : (volume ?? 1),
  });
  useInputStreamInOfflineContext(inputId);
  return createElement(InnerInputStream, { ...otherProps, inputId: `global:${inputId}` });
}

function useInputStreamInOfflineContext(inputId: string) {
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

function sceneBuilder(props: InputStreamProps, _children: SceneComponent[]): Api.Component {
  return {
    type: 'input_stream',
    id: props.id,
    input_id: props.inputId,
  };
}

export default InputStream;
