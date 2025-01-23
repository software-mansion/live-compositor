import type { MP4ArrayBuffer, MP4File, MP4Info, Sample } from 'mp4box';
import MP4Box, { DataStream } from 'mp4box';
import type { ContainerInfo, EncodedVideoPayload, EncodedVideoSource } from '../input';
import type { Logger } from 'pino';
import { Queue } from '@datastructures-js/queue';
import type { Framerate } from '../../../compositor/compositor';
import { assert } from '../../../utils';

export type Mp4Metadata = {
  video: {
    decoderConfig: VideoDecoderConfig;
    framerate: Framerate;
    trackId: number;
    frameCount: number;
    durationMs: number;
  };
};

export class Mp4Demuxer implements EncodedVideoSource {
  private file: MP4File;
  private logger: Logger;
  private ptsOffset?: number;

  private videoChunks: Queue<EncodedVideoChunk>;
  private videoTrackFinished: boolean = false;

  private readyPromise: Promise<Mp4Metadata>;
  private firstVideoChunkPromise: Promise<void>;
  private mp4Metadata?: Mp4Metadata;

  public constructor(data: ArrayBuffer, logger: Logger) {
    this.logger = logger;
    this.videoChunks = new Queue();

    this.file = MP4Box.createFile();
    this.readyPromise = new Promise<Mp4Metadata>((res, rej) => {
      this.file.onReady = info => {
        try {
          res(this.parseMp4Info(info));
        } catch (err: any) {
          rej(err);
        }
      };
      this.file.onError = (error: string) => {
        this.logger.error(`MP4Demuxer error: ${error}`);
        rej(new Error(error));
      };
    });

    let firstVideoChunkCb: (() => void) | undefined;
    this.firstVideoChunkPromise = new Promise<void>((res, _rej) => {
      firstVideoChunkCb = res;
    });

    this.file.onSamples = (id, _user, samples) => {
      this.onSamples(samples);
      if (id === this.mp4Metadata?.video.trackId) {
        firstVideoChunkCb?.();
      }
    };

    const mp4Data = data as MP4ArrayBuffer;
    mp4Data.fileStart = 0;

    this.file.appendBuffer(mp4Data);
  }

  public async init(): Promise<void> {
    this.mp4Metadata = await this.readyPromise;
    this.file.setExtractionOptions(this.mp4Metadata.video.trackId);
    this.file.start();

    // by flushing we are signaling that there won't be any new
    // chunks added
    this.file.flush();

    await this.firstVideoChunkPromise;
  }

  public getMetadata(): ContainerInfo {
    assert(this.mp4Metadata, 'Mp4 metadata not available, call `init` first.');
    return {
      video: {
        durationMs: this.mp4Metadata.video.durationMs,
        decoderConfig: this.mp4Metadata.video.decoderConfig,
      },
    };
  }

  public nextChunk(): EncodedVideoPayload | undefined {
    const chunk = this.videoChunks.pop();
    if (chunk) {
      return { type: 'chunk', chunk };
    } else if (this.videoTrackFinished) {
      return { type: 'eos' };
    }
    return;
  }

  public close(): void {
    this.file.stop();
  }

  private parseMp4Info(info: MP4Info): Mp4Metadata {
    if (info.videoTracks.length == 0) {
      throw new Error('No video tracks');
    }

    const videoTrack = info.videoTracks[0];
    const videoDurationMs = (videoTrack.movie_duration / videoTrack.movie_timescale) * 1000;
    const codecDescription = this.getCodecDescription(videoTrack.id);
    const frameCount = videoTrack.nb_samples;

    const decoderConfig = {
      codec: videoTrack.codec,
      codedWidth: videoTrack.video.width,
      codedHeight: videoTrack.video.height,
      description: codecDescription,
    };
    const framerate = {
      num: videoTrack.timescale,
      den: 1000,
    };

    return {
      video: {
        decoderConfig,
        framerate,
        trackId: videoTrack.id,
        frameCount,
        durationMs: videoDurationMs,
      },
    };
  }

  private onSamples(samples: Sample[]) {
    assert(this.mp4Metadata);

    for (const sample of samples) {
      const pts = (sample.cts * 1_000_000) / sample.timescale;
      if (this.ptsOffset === undefined) {
        this.ptsOffset = -pts;
      }

      const chunk = new EncodedVideoChunk({
        type: sample.is_sync ? 'key' : 'delta',
        timestamp: pts + this.ptsOffset,
        duration: (sample.duration * 1_000_000) / sample.timescale,
        data: sample.data,
      });

      this.videoChunks.push(chunk);

      if (sample.number === this.mp4Metadata.video.frameCount - 1) {
        this.videoTrackFinished = true;
      }
    }
  }

  private getCodecDescription(trackId: number): Uint8Array {
    const track = this.file.getTrackById(trackId);
    if (!track) {
      throw new Error('Track does not exist');
    }

    for (const entry of track.mdia.minf.stbl.stsd.entries) {
      const box = entry.avcC || entry.hvcC || entry.vpcC || entry.av1C;
      if (box) {
        const stream = new DataStream(undefined, 0, DataStream.BIG_ENDIAN);
        box.write(stream);
        return new Uint8Array(stream.buffer, 8);
      }
    }

    throw new Error('Codec description not found');
  }
}
