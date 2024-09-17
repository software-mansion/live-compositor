import { MP4ArrayBuffer } from 'mp4box';
import { MP4Decoder } from './mp4/decoder';
import { FrameFormat, FrameSet } from '@live-compositor/browser-render';
import { useEffect, useRef } from 'react';
import { useRenderer } from './utils';

const BUNNY_URL = 'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4';

function MP4Player() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const renderer = useRenderer();

  useEffect(() => {
    if (renderer == null) {
      return;
    }

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

    const decoder = new MP4Decoder();
    fetch(BUNNY_URL)
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

        decoder.decode(videoData as MP4ArrayBuffer);
      });

    const canvas = canvasRef!.current!;
    const ctx = canvas.getContext('2d');

    let pts = 0;
    const renderInterval = setInterval(async () => {
      const inputs: FrameSet = {
        ptsMs: pts,
        frames: {},
      };

      const frame = decoder.nextFrame();
      if (frame) {
        const frameOptions = {
          format: 'RGBA',
        };
        const buffer = new Uint8ClampedArray(
          frame.allocationSize(frameOptions as VideoFrameCopyToOptions)
        );
        await frame.copyTo(buffer, frameOptions as VideoFrameCopyToOptions);

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

    return () => clearInterval(renderInterval);
  }, [renderer])

  return (
    <>
      <div className="card">
        <canvas ref={canvasRef} width={1280} height={720}></canvas>
      </div>
    </>
  );
}

export default MP4Player;
