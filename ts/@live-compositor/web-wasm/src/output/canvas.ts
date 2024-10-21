import { Frame } from '@live-compositor/browser-render';
import { OutputSink } from './sink';

export default class CanvasSink implements OutputSink {
  private ctx: CanvasRenderingContext2D;

  public constructor(canvas: HTMLCanvasElement) {
    this.ctx = canvas.getContext('2d')!;
  }

  public async send(frame: Frame): Promise<void> {
    const resolution = frame.resolution;
    this.ctx.putImageData(new ImageData(frame.data, resolution.width, resolution.height), 0, 0);
  }
}
