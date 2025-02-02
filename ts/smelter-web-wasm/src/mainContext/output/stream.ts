import type { RegisterOutput } from '../../workerApi';
import type { AudioMixer } from '../AudioMixer';
import { MediaStreamAudioMixer } from '../AudioMixer';
import type { Output, RegisterOutputResult, RegisterWasmStreamOutput } from '../output';

type StreamOptions = {
  stream: MediaStream;
  mixer?: AudioMixer;
};

export class StreamOutput implements Output {
  private stream: MediaStream;
  private mixer?: AudioMixer;

  constructor(options: StreamOptions) {
    this.stream = options.stream;
    this.mixer = options.mixer;
  }

  public get audioMixer(): AudioMixer | undefined {
    return this.mixer;
  }

  public async terminate(): Promise<void> {
    this.stream.getTracks().forEach(track => track.stop());
    await this.mixer?.close();
  }
}

export async function handleRegisterStreamOutput(
  outputId: string,
  request: RegisterWasmStreamOutput
): Promise<RegisterOutputResult> {
  let video: RegisterOutput['video'] = undefined;
  let stream = new MediaStream();
  let transferable: Transferable[] = [];

  if (request.video && request.initial.video) {
    const canvas = document.createElement('canvas');
    canvas.width = request.video.resolution.width;
    canvas.height = request.video.resolution.height;
    const stream = canvas.captureStream(60);
    const track = stream.getVideoTracks()[0];
    const offscreen = canvas.transferControlToOffscreen();

    transferable.push(canvas);

    stream.addTrack(track);

    video = {
      resolution: request.video.resolution,
      initial: request.initial.video,
      canvas: offscreen,
    };
  }

  let mixer: MediaStreamAudioMixer | undefined;
  if (request.audio) {
    mixer = new MediaStreamAudioMixer();
    stream.addTrack(mixer.outputMediaStreamTrack());
  }

  const output = new StreamOutput({ stream, mixer });

  return {
    output,
    result: {
      type: 'web-wasm-stream',
      stream,
    },
    workerMessage: [
      {
        type: 'registerOutput',
        outputId,
        output: {
          type: 'stream',
          video,
        },
      },
      transferable,
    ],
  };
}
