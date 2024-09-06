import React from 'react';
import * as Api from '../api';
import { createCompositorComponent, SceneComponent } from '../component';
import { useAudioInput } from '../hooks';

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
  mute?: boolean;
};

type AudioPropNames = 'mute' | 'volume' | 'disableAudioControl';

const InnerInputStream =
  createCompositorComponent<Omit<InputStreamProps, AudioPropNames>>(sceneBuilder);

function InputStream(props: InputStreamProps) {
  const { mute, volume, ...otherProps } = props;
  useAudioInput(props.inputId, {
    volume: mute ? 0 : (volume ?? 1),
  });
  return React.createElement(InnerInputStream, otherProps);
}

function sceneBuilder(props: InputStreamProps, _children: SceneComponent[]): Api.Component {
  return {
    type: 'input_stream',
    id: props.id,
    input_id: props.inputId,
  };
}

export default InputStream;