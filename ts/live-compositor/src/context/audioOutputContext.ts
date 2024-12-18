import type { InputRef } from '../types/inputRef.js';
import { areInputRefsEqual } from '../types/inputRef.js';

export type ContextAudioOptions = {
  volume: number;
};

export type AudioMixerState = AudioInputConfig[];

export type AudioInputConfig = {
  inputRef: InputRef;
  volumeComponents: ContextAudioOptions[];
};

export type AudioConfig = Array<{ inputRef: InputRef; volume: number }>;

export class AudioContext {
  private audioMixerConfig: AudioMixerState;
  private onChange: () => void;

  constructor(onChange: () => void) {
    this.audioMixerConfig = [];
    this.onChange = onChange;
  }

  public getAudioConfig(): AudioConfig {
    return this.audioMixerConfig.map(input => ({
      inputRef: input.inputRef,
      volume: Math.min(
        input.volumeComponents.reduce((acc, opt) => acc + opt.volume, 0),
        1.0
      ),
    }));
  }

  public addInputAudioComponent(inputRef: InputRef, options: ContextAudioOptions) {
    const inputConfig = this.audioMixerConfig.find(input =>
      areInputRefsEqual(input.inputRef, inputRef)
    );
    if (inputConfig) {
      inputConfig.volumeComponents = [...inputConfig.volumeComponents, options];
    } else {
      this.audioMixerConfig = [
        ...this.audioMixerConfig,
        {
          inputRef,
          volumeComponents: [options],
        },
      ];
    }
    this.onChange();
  }

  public removeInputAudioComponent(inputRef: InputRef, options: ContextAudioOptions) {
    const inputConfig = this.audioMixerConfig.find(input =>
      areInputRefsEqual(input.inputRef, inputRef)
    );
    if (inputConfig) {
      // opt !== options compares objects by reference
      inputConfig.volumeComponents = inputConfig.volumeComponents.filter(opt => opt !== options);
      this.onChange();
    }
  }
}
