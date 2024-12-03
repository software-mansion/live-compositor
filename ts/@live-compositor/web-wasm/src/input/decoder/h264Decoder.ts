import { Queue } from '@datastructures-js/queue';

const MAX_DECODED_FRAMES = 3;

export class H264Decoder {
  private chunks: Queue<EncodedVideoChunk>;
  private frames: Queue<VideoFrame>;
  private decoder: VideoDecoder;

  public constructor() {
    this.chunks = new Queue();
    this.frames = new Queue();
    // TODO(noituri): Use web workers
    this.decoder = new VideoDecoder({
      output: frame => {
        this.frames.push(frame);
      },
      error: error => {
        console.error(`MP4Decoder error: ${error}`);
      },
    });
  }

  public configure(config: VideoDecoderConfig) {
    this.decoder.configure(config);
  }

  public enqueueChunk(chunk: EncodedVideoChunk) {
    this.chunks.push(chunk);
    this.decodeChunks();
  }

  /**
   * Returns decoded video frames. Frames have to be manually freed from memory
   */
  public getFrame(): VideoFrame | undefined {
    this.decodeChunks();
    return this.frames.pop();
  }

  private decodeChunks() {
    while (
      this.frames.size() < MAX_DECODED_FRAMES &&
      this.decoder.decodeQueueSize < MAX_DECODED_FRAMES
    ) {
      const chunk = this.chunks.pop();
      if (!chunk) {
        break;
      }
      this.decoder.decode(chunk);
    }
  }
}
