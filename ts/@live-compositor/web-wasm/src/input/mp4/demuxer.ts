import type { MP4ArrayBuffer, MP4File, MP4Info, Sample } from 'mp4box';
import MP4Box, { DataStream } from 'mp4box';
import type { SourcePayload } from '../source';
import { assert } from '../../utils';
import type { Framerate } from '../../compositor';

export type Mp4ReadyData = {
  decoderConfig: VideoDecoderConfig;
  framerate: Framerate;
};

export type MP4DemuxerCallbacks = {
  onReady: (data: Mp4ReadyData) => void;
  onPayload: (payload: SourcePayload) => void;
};

export class MP4Demuxer {
  private file: MP4File;
  private fileOffset: number;
  private callbacks: MP4DemuxerCallbacks;
  private samplesCount?: number;
  private ptsOffset?: number;

  public constructor(callbacks: MP4DemuxerCallbacks) {
    this.file = MP4Box.createFile();
    this.file.onReady = info => this.onReady(info);
    this.file.onSamples = (_id, _user, samples) => this.onSamples(samples);
    this.file.onError = (error: string) => {
      console.error(`MP4Demuxer error: ${error}`);
    };
    this.fileOffset = 0;

    this.callbacks = callbacks;
  }

  public demux(data: ArrayBuffer) {
    const mp4Data = data as MP4ArrayBuffer;
    mp4Data.fileStart = this.fileOffset;
    this.fileOffset += mp4Data.byteLength;

    this.file.appendBuffer(mp4Data);
  }

  public flush() {
    this.file.flush();
  }

  private onReady(info: MP4Info) {
    if (info.videoTracks.length == 0) {
      throw new Error('No video tracks');
    }

    const videoTrack = info.videoTracks[0];
    const codecDescription = this.getCodecDescription(videoTrack.id);
    this.samplesCount = videoTrack.nb_samples;

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

    this.callbacks.onReady({
      decoderConfig,
      framerate,
    });

    this.file.setExtractionOptions(videoTrack.id);
    this.file.start();
  }

  private onSamples(samples: Sample[]) {
    assert(this.samplesCount !== undefined);

    for (const sample of samples) {
      const pts = (sample.cts * 1_000) / sample.timescale;
      if (this.ptsOffset === undefined) {
        this.ptsOffset = -pts;
      }

      const chunk = new EncodedVideoChunk({
        type: sample.is_sync ? 'key' : 'delta',
        timestamp: pts + this.ptsOffset,
        duration: (sample.duration * 1_000) / sample.timescale,
        data: sample.data,
      });

      this.callbacks.onPayload({ type: 'chunk', chunk: chunk });

      if (sample.number === this.samplesCount - 1) {
        this.callbacks.onPayload({ type: 'eos' });
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
