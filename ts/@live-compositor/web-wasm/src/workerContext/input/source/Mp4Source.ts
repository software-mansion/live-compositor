import { Mp4Demuxer } from './Mp4Demuxer';
import type {
  ContainerInfo,
  InputStartResult,
  InputVideoFrameSource,
  VideoFramePayload,
} from '../input';
import { Decoder } from '../decoder';
import type { Logger } from 'pino';
import { assert } from '../../../utils';

export default class Mp4Source implements InputVideoFrameSource {
  private fileUrl: string;
  private logger: Logger;
  private decoder?: Decoder;
  private metadata?: ContainerInfo;

  public constructor(fileUrl: string, logger: Logger) {
    this.fileUrl = fileUrl;
    this.logger = logger;
  }

  public async init(): Promise<void> {
    const resp = await fetch(this.fileUrl);
    const fileData = await resp.arrayBuffer();

    const demuxer = new Mp4Demuxer(fileData, this.logger);
    await demuxer.init();

    this.decoder = new Decoder(demuxer, this.logger);
    await this.decoder.init();

    this.metadata = demuxer.getMetadata();
  }

  public nextFrame(): VideoFramePayload | undefined {
    assert(this.decoder, 'Decoder was not initialized, call init() first.');
    return this.decoder.nextFrame();
  }

  public getMetadata(): InputStartResult {
    return {
      videoDurationMs: this.metadata?.video.durationMs,
    };
  }

  public close(): void {
    assert(this.decoder, 'Decoder was not initialized, call init() first.');
    this.decoder.close();
  }
}
