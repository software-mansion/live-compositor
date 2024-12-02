export type FrameWithPts = {
  frame: Omit<VideoFrame, 'timestamp'>;
  ptsMs: number;
};

export type H264DecoderProps = {
  onFrame: (frame: FrameWithPts) => void;
};

export class H264Decoder {
  private decoder: VideoDecoder;
  private ptsOffset?: number;

  public constructor(props: H264DecoderProps) {
    // TODO(noituri): Use web workers
    this.decoder = new VideoDecoder({
      output: videoFrame => props.onFrame(this.intoFrameWithPts(videoFrame)),
      error: error => {
        console.error(`H264Decoder error: ${error}`);
      },
    });
  }

  public configure(config: VideoDecoderConfig) {
    this.decoder.configure(config);
  }

  public decode(chunk: EncodedVideoChunk) {
    this.decoder.decode(chunk);
  }

  public decodeQueueSize(): number {
    return this.decoder.decodeQueueSize;
  }

  private intoFrameWithPts(videoFrame: VideoFrame): FrameWithPts {
    if (this.ptsOffset === undefined) {
      this.ptsOffset = -videoFrame.timestamp;
    }

    return {
      frame: videoFrame,
      // TODO(noituri): Handle pts roller
      ptsMs: (this.ptsOffset + videoFrame.timestamp) / 1000,
    };
  }

  public async flush(): Promise<void> {
    await this.decoder.flush();
  }

  public close() {
    this.decoder.close();
  }
}
