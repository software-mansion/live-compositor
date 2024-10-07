import { Queue } from '@datastructures-js/queue';
import { VideoPayload } from '../payload';
import { InputFrame } from '../input';
import { FrameFormat } from '@live-compositor/browser-render';

export const MAX_DECODED_FRAMES = 3;

export class H264Decoder {
  private decodeQueue: Queue<VideoPayload>;
  private frames: Queue<VideoFrame>;
  private decoder: VideoDecoder;
  private ptsOffset?: number;
  private frameFormat: VideoPixelFormat;
  private eosReceived: boolean = false;

  public constructor() {
    this.decodeQueue = new Queue();
    this.frames = new Queue();
    this.decoder = new VideoDecoder({
      output: frame => {
        this.frames.push(frame);
      },
      error: error => {
        console.error(`MP4Decoder error: ${error}`);
      },
    });

    // Safari does not support conversion to RGBA
    // Chrome does not support conversion to YUV
    const isSafari = !!(window as any).safari;
    this.frameFormat = isSafari ? 'I420' : 'RGBA';
  }

  public configure(config: VideoDecoderConfig) {
    this.decoder.configure(config);
  }

  public enqueue(payload: VideoPayload) {
    if (this.eosReceived) {
      console.warn('Already closed decoder received payload');
    }

    this.decodeQueue.push(payload);
    this.decodeChunks();
  }

  /**
   * Returns decoded video frames. Frames have to be manually freed from memory
   */
  public async getFrame(): Promise<InputFrame | undefined> {
    this.decodeChunks();
    const frame = this.frames.pop();
    if (!frame) {
      return undefined;
    }

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

  /**
   * Returns `true` when all of the decoder's work has finished
   */
  public isFinished(): boolean {
    return this.frames.isEmpty() && this.decoder.decodeQueueSize == 0 && this.eosReceived;
  }

  public getBufferLength(): number {
    return this.frames.size();
  }

  private decodeChunks() {
    while (
      this.frames.size() < MAX_DECODED_FRAMES &&
      this.decoder.decodeQueueSize < MAX_DECODED_FRAMES
    ) {
      const payload = this.decodeQueue.pop();
      if (!payload) {
        break;
      }

      if (payload.type == 'chunk') {
        this.decoder.decode(payload.chunk);
      } else if (payload.type == 'eos') {
        this.eosReceived = true;
      }
    }
  }
}
