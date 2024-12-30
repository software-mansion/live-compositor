import { useCallback, useEffect, useState } from 'react';
import { LiveCompositor } from '@live-compositor/web-wasm';
import { InputStream, Text, View } from 'live-compositor';

function ScreenCapture() {
  const [compositor, canvasRef] = useCompositor();

  useEffect(() => {
    if (compositor == null) {
      return;
    }

    void compositor.start();
    return () => compositor.stop();
  }, [compositor]);

  return (
    <>
      <div className="card">
        <canvas ref={canvasRef} width={1280} height={720}></canvas>
      </div>
    </>
  );
}

function Scene() {
  return (
    <View style={{ width: 1280, height: 720 }}>
      <View style={{ top: 0, left: 200 }}>
        <InputStream inputId="screenCapture" />
      </View>
      <View style={{ width: 200, height: 40, backgroundColor: '#000000', bottom: 20, left: 520 }}>
        <Text style={{ fontSize: 30, fontFamily: 'Noto Sans' }}>Screen capture input</Text>
      </View>
    </View>
  );
}

function useCompositor(): [LiveCompositor | undefined, (canvas: HTMLCanvasElement) => void] {
  const [compositor, setCompositor] = useState<LiveCompositor | undefined>(undefined);
  const canvasRef = useCallback(async (canvas: HTMLCanvasElement) => {
    if (!canvas) {
      return;
    }

    const compositor = new LiveCompositor({
      framerate: {
        num: 30,
        den: 1,
      },
      streamFallbackTimeoutMs: 500,
    });

    await compositor.init();

    setCompositor(compositor);

    await compositor.registerFont(
      'https://fonts.gstatic.com/s/notosans/v36/o-0mIpQlx3QUlC5A4PNB6Ryti20_6n1iPHjcz6L1SoM-jCpoiyD9A-9a6Vc.ttf'
    );
    void compositor.registerInput('screenCapture', { type: 'screen_capture' });
    await compositor.registerOutput('output', <Scene />, {
      type: 'canvas',
      canvas: canvas,
      resolution: {
        width: 1280,
        height: 720,
      },
    });
  }, []);

  return [compositor, canvasRef];
}

export default ScreenCapture;
