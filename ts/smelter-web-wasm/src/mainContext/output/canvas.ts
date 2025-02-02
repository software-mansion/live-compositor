import type { Output, RegisterOutputResult, RegisterWasmCanvasOutput } from '../output';
import type { RegisterOutput } from '../../workerApi';
import type { AudioMixer } from '../AudioMixer';
import { PlaybackAudioMixer } from '../AudioMixer';

export class CanvasOutput implements Output {
  private mixer?: PlaybackAudioMixer;

  constructor(mixer?: PlaybackAudioMixer) {
    this.mixer = mixer;
  }

  public async terminate(): Promise<void> {
    await this.mixer?.close();
  }

  public get audioMixer(): AudioMixer | undefined {
    return this.mixer;
  }
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
    const offscreen = canvas.transferControlToOffscreen();

    transferable.push(offscreen);
    video = {
      resolution: request.video.resolution,
      initial: request.initial.video,
      canvas: offscreen,
    };
  }

  const output = new CanvasOutput(request.audio ? new PlaybackAudioMixer() : undefined);
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
