import { MP4Demuxer } from './demuxer';
import InputSource, { SourceCallbacks } from '../source';

export default class MP4Source implements InputSource {
  private fileUrl: string;
  private demuxer: MP4Demuxer;
  private callbacks?: SourceCallbacks;

  public constructor(fileUrl: string) {
    this.fileUrl = fileUrl;
    this.demuxer = new MP4Demuxer({
      onConfig: config => this.callbacks?.onDecoderConfig(config),
      onPayload: payload => this.callbacks?.onPayload(payload),
    });
  }

  public async start(): Promise<void> {
    const resp = await fetch(this.fileUrl);
    await resp.body?.pipeTo(this.sink());
  }

  public registerCallbacks(callbacks: SourceCallbacks): void {
    this.callbacks = callbacks;
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
