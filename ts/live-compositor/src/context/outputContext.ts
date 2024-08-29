import { AudioInputsConfiguration } from '../types/registerOutput';

export type ContextAudioOptions = {
  volume: number;
};

export class OutputContext {
  private audioMixerConfig?: AudioInputsConfiguration;
  private supportsAudio: boolean;
  private onChange: () => void;

  constructor(onChange: () => void, initialAudioConfig?: AudioInputsConfiguration) {
    this.audioMixerConfig = initialAudioConfig;
    this.supportsAudio = !!initialAudioConfig;
    this.onChange = onChange;
  }

  public getAudioConfig(): AudioInputsConfiguration | undefined {
    return this.audioMixerConfig;
  }

  public configureInputAudio(inputId: string, audioOptions: ContextAudioOptions) {
    if (!this.supportsAudio) {
      return;
    }
    if (!this.audioMixerConfig) {
      this.audioMixerConfig = {
        inputs: [{ inputId, volume: audioOptions.volume }],
      };
      return;
    }

    const oldInput = this.audioMixerConfig.inputs.find(input => input.inputId === inputId);

    const shouldUpdate = oldInput && oldInput.volume !== audioOptions.volume;

    if (!oldInput) {
      this.audioMixerConfig = {
        ...this.audioMixerConfig,
        inputs: [
          ...this.audioMixerConfig.inputs,
          {
            inputId,
            volume: audioOptions.volume,
          },
        ],
      };
      this.onChange();
    } else if (shouldUpdate) {
      this.audioMixerConfig = {
        ...this.audioMixerConfig,
        inputs: this.audioMixerConfig.inputs.map(input =>
          input.inputId === inputId ? { ...input, volume: audioOptions.volume } : input
        ),
      };
      this.onChange();
    }
  }
}
