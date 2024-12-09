import { assert, framerateToDurationMs } from "../../utils";
import { H264Decoder } from "../decoder/h264Decoder";
import { FrameRef } from "../frame";
import InputFrameProducer, { InputFrameProducerCallbacks } from "../inputFrameProducer";
import InputSource from "../source";

const MAX_BUFFERING_SIZE = 3;

export default class DecodingFrameProducer implements InputFrameProducer {
  private source: InputSource;
  private decoder: H264Decoder;
  private callbacks?: InputFrameProducerCallbacks;

  public constructor(source: InputSource) {
    this.source = source;
    this.decoder = new H264Decoder({
      onFrame: frame => this.callbacks?.onFrame(new FrameRef(frame)),
    })

    this.source.registerCallbacks({
      onDecoderConfig: config => this.decoder.configure(config),
    });
  }

  public async init(): Promise<void> {
    await this.source.init();
  }

  public start(): void {
    this.source.start();
  }

  public maxBufferSize(): number {
    return MAX_BUFFERING_SIZE;
  }

  public registerCallbacks(callbacks: InputFrameProducerCallbacks): void {
    this.callbacks = callbacks;
  }

  public async produce(framePts?: number): Promise<void> {
    if (!framePts) {
      this.tryEnqueueChunk();
      return;
    }
    this.enqueueChunks(framePts);

    // No more chunks will be produced. Flush all the remaining frames from the decoder
    if (this.source.isFinished() && this.decoder.decodeQueueSize() !== 0) {
      await this.decoder.flush();
    }
  }

  public isFinished(): boolean {
    return this.source.isFinished() && this.decoder.decodeQueueSize() === 0;
  }

  public close(): void {
    this.decoder.close();
  }

  private tryEnqueueChunk() {
    const chunk = this.source.nextChunk();
    if (chunk) {
      this.decoder.decode(chunk);
    }
  }

  private enqueueChunks(framePts: number) {
    const framerate = this.source.getFramerate();
    assert(framerate);

    const frameDuration = framerateToDurationMs(framerate);
    const targetPts = framePts + frameDuration * MAX_BUFFERING_SIZE;

    let chunk = this.source.peekChunk();
    while (chunk && chunk.timestamp <= targetPts) {
      this.decoder.decode(chunk);
      this.source.nextChunk();
      chunk = this.source.peekChunk();
    }
  }
}
