import { assert } from '../../utils';
import type { FrameRef } from '../frame';
import { NonCopyableFrameRef } from '../frame';
import type { InputFrameProducerCallbacks } from '../inputFrameProducer';
import type InputFrameProducer from '../inputFrameProducer';

export default class CameraFrameProducer implements InputFrameProducer {
  private stream?: MediaStream;
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
    this.stream = await navigator.mediaDevices.getUserMedia({ video: true });
  }

  public start(): void {
    assert(this.stream);

    const tracks = this.stream.getVideoTracks();
    if (tracks.length === 0) {
      throw new Error('No camera available');
    }

    // TODO(noituri): Implement backward compabilty with firefox
    const trackProcessor = new window.MediaStreamTrackProcessor<VideoFrame>({
      track: tracks[0],
    });
    this.reader = trackProcessor.readable.getReader();
  }

  public registerCallbacks(callbacks: InputFrameProducerCallbacks): void {
    this.callbacks = callbacks;
  }

  public async produce(_framePts?: number): Promise<void> {
    if (this.eosReceived) {
      return;
    }

    await this.produceFrame();
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

  private async produceFrame(): Promise<void> {
    assert(this.reader);

    const { done, value: videoFrame } = await this.reader.read();
    if (done) {
      this.eosReceived = true;
      return;
    }

    if (this.ptsOffset === undefined) {
      this.ptsOffset = -videoFrame.timestamp;
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
    return this.eosReceived;
  }

  public close(): void {
    if (!this.stream) {
      return;
    }
    for (const track of this.stream.getTracks()) {
      track.stop();
    }
  }
}
