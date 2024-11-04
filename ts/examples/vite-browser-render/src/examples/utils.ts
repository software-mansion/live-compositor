import { loadWasmModule, Renderer } from '@live-compositor/browser-render';
import { useEffect, useState } from 'react';

export function useRenderer(): Renderer | null {
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
      await renderer.registerFont(
        'https://fonts.gstatic.com/s/notosans/v36/o-0mIpQlx3QUlC5A4PNB6Ryti20_6n1iPHjcz6L1SoM-jCpoiyD9A-9a6Vc.ttf'
      );

      setRenderer(renderer);
    };

    setupRenderer().catch(err => console.error(err));
  }, []);

  return renderer;
}
