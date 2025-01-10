import { MP4Demuxer } from './demuxer';
import type InputSource from '../source';
import type { InputSourceCallbacks, SourceMetadata, SourcePayload } from '../source';
import { Queue } from '@datastructures-js/queue';

export default class MP4Source implements InputSource {
  private fileUrl: string;
  private fileData?: ArrayBuffer;
  private demuxer?: MP4Demuxer;
  private callbacks?: InputSourceCallbacks;
  private chunks: Queue<EncodedVideoChunk>;
  private eosReceived: boolean = false;
  private metadata: SourceMetadata = {};

  public constructor(fileUrl: string) {
    this.fileUrl = fileUrl;
    this.chunks = new Queue();
  }

  public async init(): Promise<void> {
    const resp = await fetch(this.fileUrl);
    this.fileData = await resp.arrayBuffer();
  }

  public async start(): Promise<void> {
    if (!this.fileData) {
      throw new Error('MP4Source has to be initialized first before processing can be started');
    }

    await new Promise<void>(resolve => {
      this.demuxer = new MP4Demuxer({
        onReady: data => {
          this.callbacks?.onDecoderConfig(data.decoderConfig);
          this.metadata.framerate = data.framerate;
          this.metadata.videoDurationMs = data.videoDurationMs;
          resolve();
        },
        onPayload: payload => this.handlePayload(payload),
      });

      this.demuxer.demux(this.fileData!);
      this.demuxer.flush();
    });
  }

  public registerCallbacks(callbacks: InputSourceCallbacks): void {
    this.callbacks = callbacks;
  }

  public isFinished(): boolean {
    return this.eosReceived && this.chunks.isEmpty();
  }

  public getMetadata(): SourceMetadata {
    return this.metadata;
  }

  public nextChunk(): EncodedVideoChunk | undefined {
    return this.chunks.pop();
  }

  public peekChunk(): EncodedVideoChunk | undefined {
    return this.chunks.front();
  }

  private handlePayload(payload: SourcePayload) {
    if (payload.type === 'chunk') {
      this.chunks.push(payload.chunk);
    } else if (payload.type === 'eos') {
      this.eosReceived = true;
    }
  }
}
