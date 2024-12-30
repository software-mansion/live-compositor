import { assert } from '../../utils';
import type { FrameRef } from '../frame';
import { NonCopyableFrameRef } from '../frame';
import type { InputFrameProducerCallbacks } from '../inputFrameProducer';
import type InputFrameProducer from '../inputFrameProducer';
import type { MediaStreamInitFn } from './mediaStreamInit';

export default class MediaStreamFrameProducer implements InputFrameProducer {
  private initMediaStream: MediaStreamInitFn;
  private stream?: MediaStream;
  private track?: MediaStreamTrack;
  private video: HTMLVideoElement;
  private ptsOffset?: number;
  private callbacks?: InputFrameProducerCallbacks;
  private prevFrame?: FrameRef;
  private onReadySent: boolean;
  private isVideoLoaded: boolean;

  public constructor(initMediaStream: MediaStreamInitFn) {
    this.initMediaStream = initMediaStream;
    this.onReadySent = false;
    this.isVideoLoaded = false;
    this.video = document.createElement('video');
  }

  public async init(): Promise<void> {
    this.stream = await this.initMediaStream();

    const tracks = this.stream.getVideoTracks();
    if (tracks.length === 0) {
      throw new Error('No video track in stream');
    }
    this.track = tracks[0];

    this.video.srcObject = this.stream;
    await new Promise(resolve => {
      this.video.onloadedmetadata = resolve;
    });

    this.isVideoLoaded = true;
  }

  public async start(): Promise<void> {
    assert(this.isVideoLoaded);
    await this.video.play();
  }

  public registerCallbacks(callbacks: InputFrameProducerCallbacks): void {
    this.callbacks = callbacks;
  }

  public async produce(_framePts?: number): Promise<void> {
    if (this.isFinished()) {
      return;
    }

    this.produceFrame();

    if (!this.onReadySent) {
      this.callbacks?.onReady();
      this.onReadySent = true;
    }
  }

  private produceFrame() {
    const videoFrame = new VideoFrame(this.video, { timestamp: performance.now() * 1000 });
    if (!this.ptsOffset) {
      this.ptsOffset = -videoFrame.timestamp;
    }

    // Only one media track video frame can be alive at the time
    if (this.prevFrame) {
      this.prevFrame.decrementRefCount();
    }
    this.prevFrame = new NonCopyableFrameRef({
      frame: videoFrame,
      ptsMs: (videoFrame.timestamp + this.ptsOffset) / 1000,
    });
  }

  public getFrameRef(_framePts: number): FrameRef | undefined {
    const frame = this.prevFrame;
    this.prevFrame = undefined;
    return frame;
  }

  public isFinished(): boolean {
    if (this.track) {
      return this.track.readyState === 'ended';
    }

    return false;
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
