import MP4Box, { DataStream, MP4ArrayBuffer, MP4File, MP4Info, Sample } from 'mp4box';

export type OnConfig = (config: VideoDecoderConfig) => void;

export type OnChunk = (chunk: EncodedVideoChunk) => void;

export class MP4Demuxer {
  private file: MP4File;
  private fileOffset: number;
  private onConfig: OnConfig;
  private onChunk: OnChunk;

  public constructor(props: { onConfig: OnConfig; onChunk: OnChunk }) {
    this.file = MP4Box.createFile();
    this.file.onReady = info => this.onReady(info);
    this.file.onSamples = (_id, _user, samples) => this.onSamples(samples);
    this.file.onError = (error: string) => {
      console.error(`MP4Demuxer error: ${error}`);
    };
    this.fileOffset = 0;

    this.onConfig = props.onConfig;
    this.onChunk = props.onChunk;
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
    const codecDescription = this.getCodecDescription(videoTrack.id);
    this.onConfig({
      codec: videoTrack.codec,
      codedWidth: videoTrack.video.width,
      codedHeight: videoTrack.video.height,
      description: codecDescription,
    });

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

      this.onChunk(chunk);
    }
  }

  private getCodecDescription(trackId: number): Uint8Array {
    const trak = this.file.getTrackById(trackId);
    if (!trak) {
      throw 'Track does not exist';
    }

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
