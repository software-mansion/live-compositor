import Fifo from '../../fifo';

const MAX_DECODED_AHEAD = 3;

// TODO(noituri): Use web workers
export class MP4Decoder {
  private chunks: Fifo<EncodedVideoChunk>;
  private frames: Fifo<VideoFrame>;
  private decoder: VideoDecoder;

  public constructor() {
    this.chunks = new Fifo();
    this.frames = new Fifo();
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

  public getFrame(): VideoFrame | undefined {
    this.decodeChunks();
    return this.frames.pop();
  }

  private decodeChunks() {
    while (
      this.frames.length < MAX_DECODED_AHEAD &&
      this.decoder.decodeQueueSize < MAX_DECODED_AHEAD
    ) {
      const chunk = this.chunks.pop();
      if (!chunk) {
        break;
      }
      this.decoder.decode(chunk);
    }
  }
}
