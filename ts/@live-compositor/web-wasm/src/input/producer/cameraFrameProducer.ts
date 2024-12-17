import { assert } from "../../utils";
import { FrameRef, NonCopyableFrameRef } from "../frame";
import InputFrameProducer, { InputFrameProducerCallbacks } from "../inputFrameProducer";

export default class CameraFrameProducer implements InputFrameProducer {
  private track?: MediaStreamVideoTrack;
  private reader?: ReadableStreamDefaultReader<VideoFrame>;
  private ptsOffset?: number;
  private onReadySent: boolean;
  private eosReceived: boolean;
  private lastFrame: NonCopyableFrameRef | undefined;
  private callbacks?: InputFrameProducerCallbacks;

  public constructor() {
    this.onReadySent = false;
    this.eosReceived = false;
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

  public registerCallbacks(callbacks: InputFrameProducerCallbacks): void {
    this.callbacks = callbacks;
  }

  public async produce(_framePts?: number): Promise<void> {
    if (this.eosReceived) {
      return;
    }

    await this.readFrame();
    if (!this.onReadySent) {
      this.callbacks?.onReady();
      this.onReadySent = true;
    }
  }


  public getFrameRef(_framePts?: number): FrameRef | undefined {
    let frame = this.lastFrame;
    this.lastFrame = undefined;
    return frame;
  }

  private async readFrame(): Promise<void> {
    assert(this.reader);

    const { done, value: videoFrame } = await this.reader.read();
    if (done) {
      this.eosReceived = true;
      return;
    }

    if (this.ptsOffset === undefined) {
      this.ptsOffset = -videoFrame.timestamp
    }

    // We can't buffer video frames from camera
    if (this.lastFrame) {
      this.lastFrame.decrementRefCount();
    }
    this.lastFrame = new NonCopyableFrameRef({
      frame: videoFrame,
      ptsMs: (videoFrame.timestamp + this.ptsOffset) / 1000,
    });
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
