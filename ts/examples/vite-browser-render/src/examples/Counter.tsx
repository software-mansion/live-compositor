import { useEffect, useRef, useState } from 'react';
import { loadWasmModule, Renderer } from '@live-compositor/browser-render';

function Counter() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [count, setCount] = useState(0);
  const renderer = useRenderer();
  useEffect(() => {
    if (renderer == null) {
      return;
    }

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
                text: `Count is ${count}`,
                align: 'center',
              },
            ],
          },
        ],
      }
    );

    let pts = 0;
    const renderInterval = setInterval(() => {
      const input = {
        ptsMs: pts,
        frames: {},
      };
      const outputs = renderer.render(input);
      const frame = outputs.frames['output'];
      const resolution = frame.resolution;
      const canvas = canvasRef.current;
      const context = canvas!.getContext('2d');
      context?.putImageData(new ImageData(frame!.data, resolution.width, resolution.height), 0, 0);

      pts += 30;
    }, 30);

    return () => clearInterval(renderInterval);
  }, [renderer, count]);

  return (
    <>
      <div className="card">
        <canvas ref={canvasRef} width={300} height={300}></canvas>
        <button onClick={() => setCount(count => count + 1)}>Count +1</button>
      </div>
    </>
  );
}

function useRenderer(): Renderer | null {
  const [renderer, setRenderer] = useState<Renderer | null>(null);
  useEffect(() => {
    const setupRenderer = async () => {
      await loadWasmModule('./assets/live-compositor.wasm');
      const renderer = await Renderer.create({
        streamFallbackTimeoutMs: 500,
      });

      await renderer.registerImage('img', {
        asset_type: 'gif',
        url: 'https://media.tenor.com/eFPFHSN4rJ8AAAAM/example.gif',
      });
      // await renderer.registerFont(
      //   'https://fonts.gstatic.com/s/notosans/v36/o-0mIpQlx3QUlC5A4PNB6Ryti20_6n1iPHjcz6L1SoM-jCpoiyD9A-9a6Vc.ttf'
      // );

      setRenderer(renderer);
    };

    setupRenderer().catch(err => console.error(err));
  }, []);

  return renderer;
}

export default Counter;
