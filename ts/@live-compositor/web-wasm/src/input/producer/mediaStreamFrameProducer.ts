import { assert } from "../../utils";
import { FrameRef, NonCopyableFrameRef } from "../frame";
import InputFrameProducer, { InputFrameProducerCallbacks } from "../inputFrameProducer";

export default class MediaStreamFrameProducer implements InputFrameProducer {
  private stream: MediaStream;
  private reader?: ReadableStreamDefaultReader<VideoFrame>;
  private ptsOffset?: number;
  private onReadySent: boolean;
  private eosReceived: boolean;
  private callbacks?: InputFrameProducerCallbacks;
  private lastFrame?: FrameRef;

  public constructor(stream: MediaStream) {
    this.stream = stream;
    this.onReadySent = false;
    this.eosReceived = false;
  }

  public async init(): Promise<void> { }

  public start(): void {
    const tracks = this.stream.getVideoTracks();
    if (tracks.length === 0) {
      throw new Error("No video track available");
    }

    // TODO(noituri): Implement backward compability with firefox
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
    return this.eosReceived;
  }

  public close(): void {
    for (const track of this.stream.getTracks()) {
      track.stop();
    }
  }
}
