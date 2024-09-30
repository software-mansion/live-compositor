import { FrameFormat } from '@live-compositor/browser-render';
import { MP4Demuxer } from './demuxer';
import { H264Decoder } from '../decoder/h264Decoder';
import { InputFrame } from '../input';
import InputSource from '../source';

export default class MP4Source implements InputSource {
  private fileUrl: string;
  private demuxer: MP4Demuxer;
  private decoder: H264Decoder;
  private frameFormat: VideoPixelFormat;

  public constructor(fileUrl: string) {
    this.fileUrl = fileUrl;
    this.demuxer = new MP4Demuxer({
      onConfig: config => this.decoder.configure(config),
      onChunk: chunk => this.decoder.enqueueChunk(chunk),
    });
    this.decoder = new H264Decoder();

    // Safari does not support conversion to RGBA
    // Chrome does not support conversion to YUV
    const isSafari = !!(window as any).safari;
    this.frameFormat = isSafari ? 'I420' : 'RGBA';
  }

  public async start(): Promise<void> {
    const resp = await fetch(this.fileUrl);
    await resp.body?.pipeTo(this.sink());
  }

  public async getFrame(): Promise<InputFrame | undefined> {
    const frame = this.decoder.getFrame();
    if (!frame) {
      return undefined;
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
      free: () => frame.close(),
    };
  }

  private sink(): WritableStream {
    return new WritableStream(
      {
        write: (fileChunk: Uint8Array) => {
          const buffer = fileChunk.buffer.slice(
            fileChunk.byteOffset,
            fileChunk.byteOffset + fileChunk.byteLength
          );
          this.demuxer.demux(buffer);
        },
        close: () => {
          this.demuxer.flush();
        },
      },
      { highWaterMark: 2 }
    );
  }
}
