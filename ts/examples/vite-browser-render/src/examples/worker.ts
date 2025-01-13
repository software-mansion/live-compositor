import { WHIPClient } from '@eyevinn/whip-web-client';
import { loadWasmModule, Renderer } from '@live-compositor/browser-render';
import { setWasmBundleUrl } from '@live-compositor/web-wasm';

setWasmBundleUrl('/assets/live-compositor.wasm');

let renderer: Renderer | undefined;

async function start(): Promise<void> {
  await loadWasmModule('/assets/live-compositor.wasm');
  renderer = await Renderer.create({
    streamFallbackTimeoutMs: 500,
  });

  await renderer.registerImage('img', {
    asset_type: 'gif',
    url: 'https://media.tenor.com/eFPFHSN4rJ8AAAAM/example.gif',
  });
  await renderer.registerFont(
    'https://fonts.gstatic.com/s/notosans/v36/o-0mIpQlx3QUlC5A4PNB6Ryti20_6n1iPHjcz6L1SoM-jCpoiyD9A-9a6Vc.ttf'
  );
  renderer.updateScene(
    'output',
    { width: 300, height: 300 },
    {
      type: 'view',
      children: [
        {
          type: 'view',
          direction: 'column',
          top: 0,
          left: 50,
          children: [
            {
              type: 'image',
              image_id: 'img',
            },
            {
              type: 'text',
              font_size: 30,
              font_family: 'Noto Sans',
              text: `Count is `,
              align: 'center',
            },
          ],
        },
      ],
    }
  );
}

self.onmessage = function (message) {
  const canvas = message.data;
  let pts = 0;
  self.postMessage('done');

  let counter = 1;

  setInterval(() => {
    console.log('update');
    counter += 1;
    renderer!.updateScene(
      'output',
      { width: 300, height: 300 },
      {
        type: 'view',
        children: [
          {
            type: 'view',
            direction: 'column',
            top: 0,
            left: 50,
            children: [
              {
                type: 'image',
                image_id: 'img',
              },
              {
                type: 'text',
                font_size: 30,
                font_family: 'Noto Sans',
                text: `Count is ${counter}`,
                align: 'center',
              },
            ],
          },
        ],
      }
    );
  }, 100);

  setInterval(() => {
    const input = {
      ptsMs: pts,
      frames: {},
    };
    pts += 30;
    const outputs = renderer!.render(input);
    const frame = outputs.frames['output'];
    const resolution = frame.resolution;

    const context = (canvas!.canvas as OffscreenCanvas).getContext('2d');
    context?.putImageData(new ImageData(frame!.data, resolution.width, resolution.height), 0, 0);
  }, 30);
};

void start();
