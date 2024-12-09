import { assert } from "../../utils";
import { FrameRef } from "../frame";
import InputFrameProducer, { InputFrameProducerCallbacks } from "../inputFrameProducer";

// Frames from camera can't be buffered because `reader` won't produce more frames until the old ones are freed from memory
const MAX_BUFFERING_SIZE = 1;

// TODO(noituri): Check what happens if we there's multiple CameraFrameProducer constructed
export default class CameraFrameProducer implements InputFrameProducer {
  private callbacks?: InputFrameProducerCallbacks;
  private track?: MediaStreamVideoTrack;
  private reader?: ReadableStreamDefaultReader<VideoFrame>;
  private ptsOffset?: number;
  private eosReceived: boolean;
  private isReadingFrame: boolean;
  private lastPts: number = 0;

  public constructor() {
    this.eosReceived = false;
    this.isReadingFrame = false;
  }

  public async init(): Promise<void> {
    const stream = await navigator.mediaDevices.getUserMedia({ video: true });
    const tracks = stream.getVideoTracks();
    if (tracks.length === 0) {
      throw new Error('No camera available');
    }

    this.track = stream.getVideoTracks()[0];
  }

  public start(): void {
    assert(this.track);
    // TODO(noituri): Implement backward compabilty with firefox
    const trackProcessor = new MediaStreamTrackProcessor({ track: this.track });
    this.reader = trackProcessor.readable.getReader();
  }

  public maxBufferSize(): number {
    return MAX_BUFFERING_SIZE;
  }

  public registerCallbacks(callbacks: InputFrameProducerCallbacks): void {
    this.callbacks = callbacks;
  }

  public async produce(_framePts?: number): Promise<void> {
    // `readFrame` may block indefinitely if previously read frames are not freed from memory
    if (this.isReadingFrame || this.eosReceived) {
      console.error('Reading');
      return;
    }

    void (async () => {
      this.isReadingFrame = true;
      await this.readFrame();
      this.isReadingFrame = false;
    })();
  }

  private async readFrame(_framePts?: number): Promise<void> {
    assert(this.reader);

    const { done, value: videoFrame } = await this.reader.read();
    if (done) {
      this.eosReceived = true;
      return;
    }

    if (this.ptsOffset === undefined) {
      this.ptsOffset = -videoFrame.timestamp
    }
    console.error(`elapsed ${(videoFrame.timestamp / 1000) - (this.lastPts / 1000)}`);
    this.lastPts = videoFrame.timestamp;
    this.callbacks?.onFrame(new FrameRef({
      frame: videoFrame,
      // TODO(noituri): Handle pts roller
      ptsMs: (videoFrame.timestamp + this.ptsOffset) / 1000,
    }));
  }

  public isFinished(): boolean {
    return this.track?.readyState === 'ended';
  }

  public close(): void {
    // TODO(noituri): Check what `releaseLock()` does
    this.reader?.releaseLock();
    // TODO(noituri): Check what `cancel()` does
    this.reader?.cancel();
    // TODO(noituri): Check what `stop()` does
    this.track?.stop();
  }
}
