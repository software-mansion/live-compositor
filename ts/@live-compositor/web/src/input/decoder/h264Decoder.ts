import { InputFrame } from '../input';
import { FrameFormat } from '@live-compositor/browser-render';
import Decoder, { DecoderCallbacks } from './decoder';

export class H264Decoder implements Decoder {
  private decoder: VideoDecoder;
  private ptsOffset?: number;
  private frameFormat: VideoPixelFormat;
  private decoderClosed: boolean = false;
  private callbacks?: DecoderCallbacks;

  public constructor() {
    this.decoder = new VideoDecoder({
      output: async frame =>
        this.callbacks?.onPayload({ type: 'frame', data: await this.intoInputFrame(frame) }),
      error: error => {
        console.error(`MP4Decoder error: ${error}`);
      },
    });

    // Safari does not support conversion to RGBA
    // Chrome does not support conversion to YUV
    const isSafari = !!(window as any).safari;
    this.frameFormat = isSafari ? 'I420' : 'RGBA';
  }

  public configure(config: VideoDecoderConfig): void {
    this.decoder.configure(config);
  }

  public registerCallbacks(callbacks: DecoderCallbacks): void {
    this.callbacks = callbacks;
  }

  public enqueue(chunk: EncodedVideoChunk): void {
    if (this.decoderClosed) {
      console.warn('Already closed decoder received payload');
    }

    this.decoder.decode(chunk);
  }

  public isClosed(): boolean {
    return this.decoderClosed;
  }

  public async close(): Promise<void> {
    if (this.decoderClosed) {
      console.warn('Decoder already closed');
      return;
    }

    this.decoderClosed = true;
    await this.decoder.flush();
    this.decoder.close();
  }

  public decodeQueueSize(): number {
    return this.decoder.decodeQueueSize;
  }

  /**
   * Returns decoded video frames. Frames have to be manually freed from memory
   */
  private async intoInputFrame(frame: VideoFrame): Promise<InputFrame> {
    const currentPts = frame.timestamp / 1000;
    if (!this.ptsOffset) {
      this.ptsOffset = -currentPts;
    }

    const options = {
      format: this.frameFormat,
    };
    const buffer = new Uint8ClampedArray(frame.allocationSize(options as VideoFrameCopyToOptions));
    await frame.copyTo(buffer, options as VideoFrameCopyToOptions);

    return {
      resolution: {
        width: frame.displayWidth,
        height: frame.displayHeight,
      },
      format: this.frameFormat == 'I420' ? FrameFormat.YUV_BYTES : FrameFormat.RGBA_BYTES,
      data: buffer,
      // TODO(noituri): This will not work if there is pts rollover
      ptsMs: currentPts + this.ptsOffset!,
      free: () => frame.close(),
    };
  }
}
