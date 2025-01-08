import { Queue } from '@datastructures-js/queue';
import { assert, framerateToDurationMs } from '../../utils';
import { H264Decoder } from '../decoder/h264Decoder';
import { FrameRef } from '../frame';
import type { InputFrameProducerCallbacks } from '../inputFrameProducer';
import type InputFrameProducer from '../inputFrameProducer';
import type InputSource from '../source';

const MAX_BUFFERING_SIZE = 3;

export default class DecodingFrameProducer implements InputFrameProducer {
  private source: InputSource;
  private decoder: H264Decoder;
  private frames: Queue<FrameRef>;
  private onReadySent: boolean;
  private callbacks?: InputFrameProducerCallbacks;

  public constructor(source: InputSource) {
    this.onReadySent = false;
    this.source = source;
    this.decoder = new H264Decoder({
      onFrame: frame => this.frames.push(new FrameRef(frame)),
    });
    this.frames = new Queue();

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

  public registerCallbacks(callbacks: InputFrameProducerCallbacks): void {
    this.callbacks = callbacks;
  }

  public async produce(framePts?: number): Promise<void> {
    if (!this.onReadySent && this.frames.size() >= MAX_BUFFERING_SIZE) {
      this.callbacks?.onReady();
      this.onReadySent = true;
    }

    if (this.source.isFinished()) {
      // No more chunks will be produced. Flush all the remaining frames from the decoder
      if (this.decoder.decodeQueueSize() !== 0) {
        await this.decoder.flush();
      }
      return;
    }

    if (framePts) {
      this.enqueueChunksForPts(framePts);
    } else {
      this.tryEnqueueChunk();
    }
  }

  /**
   * Retrieves frame with PTS closest to `framePts`.
   * Frames older than the closest frame are dropped.
   */
  public getFrameRef(framePts: number): FrameRef | undefined {
    this.dropOldFrames(framePts);

    if (
      this.source.isFinished() &&
      this.decoder.decodeQueueSize() === 0 &&
      this.frames.size() == 1
    ) {
      return this.frames.pop();
    } else {
      return this.cloneLatestFrame();
    }
  }

  /**
   * Retrieves latest frame and increments its reference count
   */
  private cloneLatestFrame(): FrameRef | undefined {
    const frame = this.frames.front();
    if (frame) {
      frame.incrementRefCount();
      return frame;
    }

    return undefined;
  }

  public isFinished(): boolean {
    return (
      this.source.isFinished() && this.decoder.decodeQueueSize() === 0 && this.frames.isEmpty()
    );
  }

  public close(): void {
    this.decoder.close();
  }

  /**
   * Finds frame with PTS closest to `framePts` and removes frames older than it
   */
  private dropOldFrames(framePts: number): void {
    if (this.frames.isEmpty()) {
      return;
    }

    const frames = this.frames.toArray();
    const targetFrame = frames.reduce((prevFrame, frame) => {
      const prevPtsDiff = Math.abs(prevFrame.getPtsMs() - framePts);
      const currPtsDiff = Math.abs(frame.getPtsMs() - framePts);
      return prevPtsDiff < currPtsDiff ? prevFrame : frame;
    });

    for (const frame of frames) {
      if (frame.getPtsMs() < targetFrame.getPtsMs()) {
        frame.decrementRefCount();
        this.frames.pop();
      }
    }
  }

  private tryEnqueueChunk() {
    const chunk = this.source.nextChunk();
    if (chunk) {
      this.decoder.decode(chunk);
    }
  }

  private enqueueChunksForPts(framePts: number) {
    const framerate = this.source.getFramerate();
    assert(framerate);

    const frameDuration = framerateToDurationMs(framerate);
    const targetPtsUs = (framePts + frameDuration * MAX_BUFFERING_SIZE) * 1000;

    let chunk = this.source.peekChunk();
    while (chunk && chunk.timestamp <= targetPtsUs) {
      this.decoder.decode(chunk);
      this.source.nextChunk();
      chunk = this.source.peekChunk();
    }
  }
}
