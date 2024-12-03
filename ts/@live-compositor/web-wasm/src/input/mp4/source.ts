import type { Mp4ReadyData } from './demuxer';
import { MP4Demuxer } from './demuxer';
import type InputSource from '../source';
import type { InputSourceCallbacks, SourcePayload } from '../source';
import { Queue } from '@datastructures-js/queue';
import type { Framerate } from '../../compositor';

export default class MP4Source implements InputSource {
  private fileUrl: string;
  private fileData?: ArrayBuffer;
  private demuxer: MP4Demuxer;
  private callbacks?: InputSourceCallbacks;
  private chunks: Queue<EncodedVideoChunk>;
  private eosReceived: boolean = false;
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

  public nextChunk(): EncodedVideoChunk | undefined {
    return this.chunks.pop();
  }

  public peekChunk(): EncodedVideoChunk | undefined {
    return this.chunks.front();
  }

  private handleOnReady(data: Mp4ReadyData) {
    this.callbacks?.onDecoderConfig(data.decoderConfig);
    this.framerate = data.framerate;
  }

  private handlePayload(payload: SourcePayload) {
    if (payload.type === 'chunk') {
      this.chunks.push(payload.chunk);
    } else if (payload.type === 'eos') {
      this.eosReceived = true;
    }
  }
}
