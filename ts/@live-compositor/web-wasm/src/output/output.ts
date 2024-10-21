import { Frame, Resolution } from '@live-compositor/browser-render';
import { OutputSink } from './sink';
import CanvasSink from './canvas';
import { RegisterOutputRequest } from '@live-compositor/core';

export class Output {
  private sink: OutputSink;
  public readonly resolution: Resolution;

  public constructor(request: RegisterOutputRequest) {
    if (request.type === 'canvas') {
      this.sink = new CanvasSink(request.video.canvas);
    } else {
      throw new Error(`Unknown output type ${(request as any).type}`);
    }
    this.resolution = request.video.resolution;
  }

  public async send(frame: Frame): Promise<void> {
    await this.sink.send(frame);
  }
}
