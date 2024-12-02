import type { Mp4ReadyData } from './demuxer';
import { MP4Demuxer } from './demuxer';
import type InputSource from '../source';
import type { InputSourceCallbacks, SourcePayload, VideoChunk } from '../source';
import { Queue } from '@datastructures-js/queue';
import { assert } from '../../utils';
import type { Framerate } from '../../compositor';

export default class MP4Source implements InputSource {
  private fileUrl: string;
  private fileData?: ArrayBuffer;
  private demuxer: MP4Demuxer;
  private callbacks?: InputSourceCallbacks;
  private chunks: Queue<EncodedVideoChunk>;
  private eosReceived: boolean = false;
  private ptsOffset?: number;
  private framerate?: Framerate;

  public constructor(fileUrl: string) {
    this.fileUrl = fileUrl;
    this.demuxer = new MP4Demuxer({
      onReady: config => this.handleOnReady(config),
      onPayload: payload => this.handlePayload(payload),
    });
    this.chunks = new Queue();
  }

  public async init(): Promise<void> {
    const resp = await fetch(this.fileUrl);
    this.fileData = await resp.arrayBuffer();
  }

  public start(): void {
    if (!this.fileData) {
      throw new Error('MP4Source has to be initialized first before processing can be started');
    }

    this.demuxer.demux(this.fileData);
    this.demuxer.flush();
  }

  public registerCallbacks(callbacks: InputSourceCallbacks): void {
    this.callbacks = callbacks;
  }

  public isFinished(): boolean {
    return this.eosReceived && this.chunks.isEmpty();
  }

  public getFramerate(): Framerate | undefined {
    return this.framerate;
  }

  public nextChunk(): VideoChunk | undefined {
    const chunk = this.chunks.pop();
    return chunk && this.intoVideoChunk(chunk);
  }

  public peekChunk(): VideoChunk | undefined {
    const chunk = this.chunks.front();
    return chunk && this.intoVideoChunk(chunk);
  }

  private handleOnReady(data: Mp4ReadyData) {
    this.callbacks?.onDecoderConfig(data.decoderConfig);
    this.framerate = data.framerate;
  }

  private handlePayload(payload: SourcePayload) {
    if (payload.type === 'chunk') {
      if (this.ptsOffset === undefined) {
        this.ptsOffset = -payload.chunk.timestamp;
      }
      this.chunks.push(payload.chunk);
    } else if (payload.type === 'eos') {
      this.eosReceived = true;
    }
  }

  private intoVideoChunk(chunk: EncodedVideoChunk): VideoChunk {
    assert(this.ptsOffset !== undefined);

    return {
      data: chunk,
      ptsMs: (this.ptsOffset + chunk.timestamp) / 1000,
    };
  }
}
