import type { RegisterOutput } from '../../workerApi';
import type { Output, RegisterOutputResult, RegisterWasmStreamOutput } from '../output';

type StreamOptions = {
  stream: MediaStream;
};

export class StreamOutput implements Output {
  private stream: MediaStream;

  constructor(options: StreamOptions) {
    this.stream = options.stream;
  }

  public async terminate(): Promise<void> {
    this.stream.getTracks().forEach(track => track.stop());
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
  } else {
    // TODO: remove after adding audio
    throw new Error('Video field is required');
  }

  // @ts-ignore
  const audioTrack = new MediaStreamTrackGenerator({ kind: 'audio' });
  stream.addTrack(audioTrack);

  const output = new StreamOutput({ stream });

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
