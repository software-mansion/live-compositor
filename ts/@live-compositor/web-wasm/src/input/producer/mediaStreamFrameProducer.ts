import { assert } from "../../utils";
import { FrameRef, NonCopyableFrameRef } from "../frame";
import InputFrameProducer, { InputFrameProducerCallbacks } from "../inputFrameProducer";

export default class MediaStreamFrameProducer implements InputFrameProducer {
  private stream: MediaStream;
  private track: MediaStreamTrack;
  private video: HTMLVideoElement;
  private canvas: HTMLCanvasElement;
  private canvasContext: CanvasRenderingContext2D;
  private ptsOffset?: number;
  private onReadySent: boolean;
  private isVideoLoaded: boolean;
  private isProducingFrame: boolean;
  private callbacks?: InputFrameProducerCallbacks;
  private lastFrame?: FrameRef;

  public constructor(stream: MediaStream) {
    this.stream = stream;
    this.onReadySent = false;
    this.isProducingFrame = false;
    this.isVideoLoaded = false;
    this.video = document.createElement('video');
    this.canvas = document.createElement('canvas');

    const canvasContext = this.canvas.getContext('2d');
    assert(canvasContext);
    this.canvasContext = canvasContext;

    const tracks = stream.getVideoTracks();
    if (tracks.length === 0) {
      throw new Error('No video track in stream');
    }

    this.track = tracks[0];
  }

  public async init(): Promise<void> {
    this.video.srcObject = this.stream;
    await new Promise((resolve) => {
      this.video.onloadedmetadata = resolve;
    });

    this.canvas.width = this.video.videoWidth;
    this.canvas.height = this.video.videoHeight;
    this.isVideoLoaded = true;
  }

  public start(): void {
    assert(this.isVideoLoaded);
    this.video.play();
  }

  public registerCallbacks(callbacks: InputFrameProducerCallbacks): void {
    this.callbacks = callbacks;
  }

  public async produce(_framePts?: number): Promise<void> {
    if (this.isProducingFrame) {
      console.error('is producing');
      return;
    }
    if (this.isFinished()) {
      return;
    }

    void (async () => {
      this.isProducingFrame = true;
      try {
        await this.produceFrame();
      } finally {
        this.isProducingFrame = false;
      }

      if (!this.onReadySent) {
        this.callbacks?.onReady();
        this.onReadySent = true;
      }
    })()
  }

  private async produceFrame(): Promise<void> {
    this.canvasContext.drawImage(this.video, 0, 0, this.canvas.width, this.canvas.height);

    const videoFrame = new VideoFrame(this.canvas, { timestamp: performance.now() * 1000 });
    if (!this.ptsOffset) {
      this.ptsOffset = -videoFrame.timestamp;
    }

    // Only one media track video frame can be alive at the time
    if (this.lastFrame) {
      this.lastFrame.decrementRefCount();
    }
    this.lastFrame = new NonCopyableFrameRef({
      frame: videoFrame,
      ptsMs: (videoFrame.timestamp + this.ptsOffset) / 1000,
    })
  }

  public getFrameRef(_framePts: number): FrameRef | undefined {
    const frame = this.lastFrame;
    this.lastFrame = undefined;
    return frame;
  }

  public isFinished(): boolean {
    return this.track.readyState === 'ended';
  }

  public close(): void {
    for (const track of this.stream.getTracks()) {
      track.stop();
    }
  }
}
