import MP4Box, { DataStream, MP4ArrayBuffer, TrakBox } from "mp4box";

export function startDecoding(videoData: MP4ArrayBuffer, onFrame: (frame: VideoFrame) => void) {
  const file = MP4Box.createFile();
  const decoder = new VideoDecoder({
    output: onFrame,
    error: error => {
      console.error(`VideoDecoder Error: ${error}`);
    },
  });

  file.onReady = info => {
    const videoTrack = info.videoTracks[0];
    console.log(`Using codec: ${videoTrack.codec}`);

    const trak = file.getTrackById(videoTrack.id);
    const description = getCodecDescription(trak);
    if (!description) {
      console.error('Codec description not found');
      return;
    }

    decoder.configure({
      codec: videoTrack.codec,
      codedWidth: videoTrack.video.width,
      codedHeight: videoTrack.video.height,
      description: description,
    });

    file.setExtractionOptions(videoTrack.id);
    file.start();
  };

  file.onSamples = (_id, _user, samples) => {
    for (const sample of samples) {
      const chunk = new EncodedVideoChunk({
        type: sample.is_sync ? 'key' : 'delta',
        timestamp: (sample.cts * 1_000_000) / sample.timescale,
        duration: (sample.duration * 1_000_000) / sample.timescale,
        data: sample.data,
      });

      decoder.decode(chunk);
    }
  };

  file.onError = (error: string) => {
    console.error(`MP4 Parser Error: ${error}`);
  };

  videoData.fileStart = 0;
  file.appendBuffer(videoData);
  file.flush();
}

function getCodecDescription(trak: TrakBox) {
  for (const entry of trak.mdia.minf.stbl.stsd.entries) {
    const box = entry.avcC || entry.hvcC || entry.vpcC || entry.av1C;
    if (box) {
      const stream = new DataStream(undefined, 0, DataStream.BIG_ENDIAN);
      box.write(stream);
      return new Uint8Array(stream.buffer, 8);
    }
  }
}
