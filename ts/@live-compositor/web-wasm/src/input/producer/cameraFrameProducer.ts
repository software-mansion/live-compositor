import { assert } from "../../utils";
import { FrameRef } from "../frame";
import InputFrameProducer, { InputFrameProducerCallbacks } from "../inputFrameProducer";

export default class CameraFrameProducer implements InputFrameProducer {
  private callbacks?: InputFrameProducerCallbacks;
  private track?: MediaStreamVideoTrack;
  private reader?: ReadableStreamDefaultReader<VideoFrame>;
  private ptsOffset?: number;

  public constructor() {
  }

  public async init(): Promise<void> {
    // TODO(noituri): Constraints should be provided in constructor
    const stream = await navigator.mediaDevices.getUserMedia({ video: true });

    // TODO(noituri): Check size
    this.track = stream.getVideoTracks()[0];
  }

  public start(): void {
    // TODO(noituri): Backward compabilty with firefox
    assert(this.track);
    const trackProcessor = new MediaStreamTrackProcessor({ track: this.track });
    this.reader = trackProcessor.readable.getReader();
  }

  public registerCallbacks(callbacks: InputFrameProducerCallbacks): void {
    this.callbacks = callbacks;
  }

  public async produce(_framePts?: number): Promise<void> {
    void this.readFrames();
  }

  private async readFrames(): Promise<void> {
    assert(this.reader);
    const { done, value: videoFrame } = await this.reader.read();
    if (done) {
      // TODO(noituri): Handle EOS
      return;
    }

    if (this.ptsOffset === undefined) {
      this.ptsOffset = -videoFrame.timestamp
    }

    // console.warn(`videoPts: ${(videoFrame.timestamp + this.ptsOffset) / 1000} < ${_framePts}`);
    this.callbacks?.onFrame(new FrameRef({
      frame: videoFrame,
      // TODO(noituri): Handle pts roller
      ptsMs: (videoFrame.timestamp + this.ptsOffset) / 1000,
    }));
  }

  public setMaxBufferSize(_maxBufferSize: number): void {
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
