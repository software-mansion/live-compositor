import type { Output, RegisterOutputResult, RegisterWasmCanvasOutput } from '../output';
import type { RegisterOutput } from '../../workerApi';

export class CanvasOutput implements Output {
  public async terminate(): Promise<void> {}
}

export async function handleRegisterCanvasOutput(
  outputId: string,
  request: RegisterWasmCanvasOutput
): Promise<RegisterOutputResult> {
  let video: RegisterOutput['video'] | undefined = undefined;
  let transferable: Transferable[] = [];

  if (request.video && request.initial.video) {
    const canvas = request.video.canvas;
    canvas.width = request.video.resolution.width;
    canvas.height = request.video.resolution.height;
    const stream = canvas.captureStream(60);
    const track = stream.getVideoTracks()[0];
    const offscreen = canvas.transferControlToOffscreen();

    stream.addTrack(track);

    transferable.push(offscreen);
    video = {
      resolution: request.video.resolution,
      initial: request.initial.video,
      canvas: offscreen,
    };
  } else {
    // TODO: remove after adding audio
    throw new Error('Video field is required');
  }

  const output = new CanvasOutput();
  return {
    output,
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
