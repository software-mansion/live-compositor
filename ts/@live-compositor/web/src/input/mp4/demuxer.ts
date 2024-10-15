import MP4Box, { DataStream, MP4ArrayBuffer, MP4File, MP4Info, Sample, TrakBox } from 'mp4box';
import { VideoPayload } from '../payload';

export type OnConfig = (config: VideoDecoderConfig) => void;
export type OnPayload = (payload: VideoPayload) => void;

export class MP4Demuxer {
  private file: MP4File;
  private fileOffset: number;
  private onConfig: OnConfig;
  private onPayload: OnPayload;
  private samplesCount?: number;

  public constructor(props: { onConfig: OnConfig; onPayload: OnPayload }) {
    this.file = MP4Box.createFile();
    this.file.onReady = info => this.onReady(info);
    this.file.onSamples = (_id, _user, samples) => this.onSamples(samples);
    this.file.onError = (error: string) => {
      console.error(`MP4Demuxer error: ${error}`);
    };
    this.fileOffset = 0;

    this.onConfig = props.onConfig;
    this.onPayload = props.onPayload;
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
      throw 'No video tracks';
    }

    const videoTrack = info.videoTracks[0];
    const trakBox = this.file.getTrackById(videoTrack.id);
    if (!trakBox) {
      throw 'Track does not exist';
    }

    const codecDescription = this.getCodecDescription(trakBox);
    this.onConfig({
      codec: videoTrack.codec,
      codedWidth: videoTrack.video.width,
      codedHeight: videoTrack.video.height,
      description: codecDescription,
    });

    this.samplesCount = trakBox.samples.length;
    this.file.setExtractionOptions(videoTrack.id);
    this.file.start();
  }

  private onSamples(samples: Sample[]) {
    for (const sample of samples) {
      const chunk = new EncodedVideoChunk({
        type: sample.is_sync ? 'key' : 'delta',
        timestamp: (sample.cts * 1_000_000) / sample.timescale,
        duration: (sample.duration * 1_000_000) / sample.timescale,
        data: sample.data,
      });

      this.onPayload({ type: 'chunk', chunk: chunk });
      if (sample.number == this.samplesCount! - 1) {
        this.onPayload({ type: 'eos' });
      }
    }
  }

  private getCodecDescription(trak: TrakBox): Uint8Array {
    for (const entry of trak.mdia.minf.stbl.stsd.entries) {
      const box = entry.avcC || entry.hvcC || entry.vpcC || entry.av1C;
      if (box) {
        const stream = new DataStream(undefined, 0, DataStream.BIG_ENDIAN);
        box.write(stream);
        return new Uint8Array(stream.buffer, 8);
      }
    }

    throw 'Codec description not found';
  }
}
