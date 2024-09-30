import { AudioInputsConfiguration } from '../types/registerOutput.js';

export type ContextAudioOptions = {
  volume: number;
};

export type AudioConfig = AudioInputConfig[];

export type AudioInputConfig = {
  inputId: string;
  volumeComponents: ContextAudioOptions[];
};

export class OutputContext {
  private audioMixerConfig?: AudioConfig;
  private onChange: () => void;

  constructor(onChange: () => void, supportsAudio: boolean) {
    this.audioMixerConfig = supportsAudio ? [] : undefined;
    this.onChange = onChange;
  }

  public getAudioConfig(): AudioInputsConfiguration | undefined {
    if (!this.audioMixerConfig) {
      return undefined;
    }

    return {
      inputs: this.audioMixerConfig.map(input => ({
        inputId: input.inputId,
        volume: Math.min(
          input.volumeComponents.reduce((acc, opt) => acc + opt.volume, 0),
          1.0
        ),
      })),
    };
  }

  public addInputAudioComponent(inputId: string, options: ContextAudioOptions) {
    if (!this.audioMixerConfig) {
      return;
    }

    const inputConfig = this.audioMixerConfig.find(input => input.inputId === inputId);
    if (inputConfig) {
      inputConfig.volumeComponents = [...inputConfig.volumeComponents, options];
    } else {
      this.audioMixerConfig = [
        ...this.audioMixerConfig,
        {
          inputId,
          volumeComponents: [options],
        },
      ];
    }
    this.onChange();
  }

  public removeInputAudioComponent(inputId: string, options: ContextAudioOptions) {
    if (!this.audioMixerConfig) {
      return;
    }

    const inputConfig = this.audioMixerConfig.find(input => input.inputId === inputId);
    if (inputConfig) {
      // opt !== options compares objects by reference
      inputConfig.volumeComponents = inputConfig.volumeComponents.filter(opt => opt !== options);
      this.onChange();
    }
  }
}
