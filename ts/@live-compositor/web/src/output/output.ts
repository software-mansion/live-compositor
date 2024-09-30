import { Frame } from '@live-compositor/browser-render';
import { OutputSink } from './sink';
import CanvasSink from './canvas';
import { RegisterOutput } from './registerOutput';

export class Output {
  private sink: OutputSink;

  public constructor(request: RegisterOutput) {
    if (request.type === 'canvas') {
      this.sink = new CanvasSink(request.canvas);
    } else {
      throw new Error(`Unknown output type ${(request as any).type}`);
    }
  }

  public async send(frame: Frame): Promise<void> {
    await this.sink.send(frame);
  }
}
