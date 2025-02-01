import type { Api } from 'live-compositor';
import { assert } from '../utils';

type AudioInput = {
  source: MediaStreamAudioSourceNode;
  gain: GainNode;
};

export class AudioMixer<OutputNode extends AudioNode = AudioNode> {
  private ctx: AudioContext;
  private inputs: Record<string, AudioInput> = {};
  protected outputNode: OutputNode;

  constructor(ctx: AudioContext, outputNode: OutputNode) {
    this.ctx = ctx;
    this.outputNode = outputNode;
  }

  public addInput(inputId: string, track: MediaStreamTrack) {
    const stream = new MediaStream();
    stream.addTrack(track);
    const source = this.ctx.createMediaStreamSource(stream);
    const gain = this.ctx.createGain();
    source.connect(gain);
    gain.connect(this.outputNode ?? this.ctx.destination);
    this.inputs[inputId] = {
      source,
      gain,
    };
  }

  public removeInput(inputId: string) {
    this.inputs[inputId]?.source.disconnect();
    this.inputs[inputId]?.gain.disconnect();
    delete this.inputs[inputId];
  }

  public update(inputConfig: Api.InputAudio[]) {
    for (const [inputId, input] of Object.entries(this.inputs)) {
      const config = inputConfig.find(input => input.input_id === inputId);
      input.gain.gain.value = config?.volume || 0;
    }
  }

  public async close() {
    await this.ctx.close();
    for (const inputId of Object.keys(this.inputs)) {
      this.removeInput(inputId);
    }
  }
}

export class MediaStreamAudioMixer extends AudioMixer<MediaStreamAudioDestinationNode> {
  constructor() {
    const ctx = new AudioContext();
    const outputNode = ctx.createMediaStreamDestination();
    const silence = ctx.createConstantSource();
    silence.offset.value = 0;
    silence.connect(outputNode);
    silence.start();
    super(ctx, outputNode);
  }

  public outputMediaStreamTrack(): MediaStreamTrack {
    const audioTrack = this.outputNode.stream.getAudioTracks()[0];
    assert(audioTrack);
    return audioTrack;
  }
}

export class PlaybackAudioMixer extends AudioMixer<AudioDestinationNode> {
  constructor() {
    const ctx = new AudioContext();
    super(ctx, ctx.destination);
  }
}
