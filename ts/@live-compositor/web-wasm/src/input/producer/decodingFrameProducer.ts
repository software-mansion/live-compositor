import { assert, framerateToDurationMs } from "../../utils";
import { H264Decoder } from "../decoder/h264Decoder";
import { FrameRef } from "../frame";
import InputFrameProducer, { DEFAULT_MAX_BUFFERING_SIZE } from "../inputFrameProducer";
import InputSource from "../source";
import { Queue } from "@datastructures-js/queue";

export default class DecodingFrameProducer implements InputFrameProducer {
  private source: InputSource;
  private decoder: H264Decoder;
  private frames: Queue<FrameRef>;
  private maxBufferSize: number;
  private isFinished: boolean;

  public constructor(source: InputSource) {
    this.source = source;
    this.maxBufferSize = DEFAULT_MAX_BUFFERING_SIZE;
    this.isFinished = false;
    this.frames = new Queue();
    this.decoder = new H264Decoder({
      onFrame: frame => this.frames.push(new FrameRef(frame)),
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

  public async produce(framePts: number): Promise<void> {
    this.enqueueChunks(framePts);

    // No more chunks will be produced. Flush all the remaining frames from the decoder
    if (this.source.isFinished() && this.decoder.decodeQueueSize() !== 0) {
      await this.decoder.flush();
    }
  }

  public getFrame(): FrameRef | undefined {
    if (this.source.isFinished() && this.frames.size() == 1) {
      this.isFinished = true;
      return this.frames.pop();
    }

    const frame = this.frames.front();
    if (frame) {
      frame.incrementRefCount();
      return frame;
    }

    return undefined;
  }

  public peekFrame(): FrameRef | undefined {
    return this.frames.front();
  }

  public frameCount(): number {
    return this.frames.size();
  }

  public setMaxBufferSize(maxBufferSize: number): void {
    this.maxBufferSize = maxBufferSize;
  }

  public isFinished(): boolean {
    return this.source.isFinished();
  }

  private enqueueChunks(framePts: number) {
    const framrate = assert(this.source.getFramerate());
    const frameDuration = framerateToDurationMs(framrate);
    const targetPts = framePts + frameDuration * this.maxBufferSize;

    let chunk = this.source.peekChunk();
    while (chunk && chunk.timestamp < targetPts) {
      this.decoder.decode(chunk);
      this.source.nextChunk();
      chunk = this.source.peekChunk();
    }
  }
}
