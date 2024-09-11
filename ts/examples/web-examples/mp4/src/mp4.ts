import MP4Box, { DataStream, MP4ArrayBuffer, TrakBox } from 'mp4box';
import { FrameFormat, FrameSet, Renderer, loadWasmModule } from '@live-compositor/browser-render';

export async function play(videoUrl: string) {
  await loadWasmModule('./assets/live-compositor.wasm');

  const frames: VideoFrame[] = [];
  const renderer = await Renderer.create({
    streamFallbackTimeoutMs: 500,
  });

  await renderer.registerFont(
    'https://fonts.gstatic.com/s/notosans/v36/o-0mIpQlx3QUlC5A4PNB6Ryti20_6n1iPHjcz6L1SoM-jCpoiyD9A-9a6Vc.ttf'
  );
  renderer.registerInput('bunny_video');
  renderer.updateScene(
    'output',
    {
      width: 1280,
      height: 720,
    },
    {
      type: 'view',
      background_color_rgba: '#000000FF',
      children: [
        {
          type: 'view',
          top: 300,
          left: 500,
          children: [
            {
              type: 'text',
              font_size: 30,
              font_family: 'Noto Sans',
              text: 'Loading MP4 file',
              align: 'right',
            },
          ],
        },
      ],
    }
  );

  fetch(videoUrl)
    .then(resp => resp.arrayBuffer())
    .then(videoData => {
      renderer.updateScene(
        'output',
        {
          width: 1280,
          height: 720,
        },
        {
          type: 'view',
          width: 1280,
          height: 720,
          background_color_rgba: '#000000FF',
          children: [
            {
              type: 'input_stream',
              input_id: 'bunny_video',
            },
            {
              type: 'view',
              width: 230,
              height: 40,
              background_color_rgba: '#000000FF',
              bottom: 20,
              left: 500,
              children: [
                {
                  type: 'text',
                  font_size: 30,
                  font_family: 'Noto Sans',
                  text: 'Playing MP4 file',
                  align: 'center',
                },
              ],
            },
          ],
        }
      );

      startDecoding(videoData as MP4ArrayBuffer, frame => {
        frames.push(frame);
      });
    });

  const canvas = document.getElementById('canvas') as HTMLCanvasElement;
  const ctx = canvas.getContext('2d');

  canvas.width = 1280;
  canvas.height = 720;

  let pts = 0;
  setInterval(() => {
    const inputs: FrameSet = {
      ptsMs: pts,
      frames: {},
    };

    const frame = frames.shift();
    if (frame) {
      const frameOptions = {
        format: 'RGBA',
      };
      const buffer = new Uint8ClampedArray(
        frame.allocationSize(frameOptions as VideoFrameCopyToOptions)
      );
      frame.copyTo(buffer, frameOptions as VideoFrameCopyToOptions);

      inputs.frames['bunny_video'] = {
        resolution: {
          width: frame.displayWidth,
          height: frame.displayHeight,
        },
        format: FrameFormat.RGBA_BYTES,
        data: buffer,
      };
    }

    const outputs = renderer.render(inputs);
    const output = outputs.frames['output'];
    ctx!.putImageData(
      new ImageData(output.data, output.resolution.width, output.resolution.height),
      0,
      0
    );

    if (frame) {
      frame.close();
    }
    pts += 30;
  }, 30);
}

function startDecoding(videoData: MP4ArrayBuffer, onFrame: (frame: VideoFrame) => void) {
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
