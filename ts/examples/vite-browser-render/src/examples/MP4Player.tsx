import { useCallback, useEffect, useState } from 'react';
import { LiveCompositor } from '@live-compositor/web-wasm';
import { InputStream, Text, useInputStreams, View } from 'live-compositor';

const BUNNY_URL = 'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4';

function MP4Player() {
  const [compositor, canvasRef] = useCompositor();

  useEffect(() => {
    if (compositor == null) {
      return;
    }

    void compositor.start();
    return () => compositor.stop();
  }, [compositor])

  return (
    <>
      <div className="card">
        <canvas ref={canvasRef} width={1280} height={720}></canvas>
      </div>
    </>
  );
}

function Scene() {
  const inputs = useInputStreams();
  const inputState = inputs['bunny_video']?.videoState;

  if (inputState !== 'playing') {
    return (
      <View backgroundColor="#000000">
        <View width={530} height={40} bottom={340} left={500}>
          <Text fontSize={30} fontFamily="Noto Sans">
            Loading MP4 file
          </Text>
        </View>
      </View>
    );
  }

  return (
    <View width={1280} height={720}>
      <InputStream inputId="bunny_video" />
      <View width={230} height={40} backgroundColor="#000000" bottom={20} left={500}>
        <Text fontSize={30} fontFamily="Noto Sans">
          Playing MP4 file
        </Text>
      </View>
    </View>
  );
}

function useCompositor(): [LiveCompositor | undefined, (canvas: HTMLCanvasElement) => void] {
  const [compositor, setCompositor] = useState<LiveCompositor | undefined>(undefined);
  const canvasRef = useCallback((canvas: HTMLCanvasElement) => {
    if (!canvas) {
      return;
    }

    const setupCompositor = async () => {
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
      void compositor.registerInput('bunny_video', { type: 'mp4', url: BUNNY_URL });
      await compositor.registerOutput('output', {
        type: 'canvas',
        canvas: canvas,
        resolution: {
          width: 1280,
          height: 720,
        },
        root: <Scene />,
      });
    }

    setupCompositor();
  }, [])

  return [compositor, canvasRef];
}

export default MP4Player;
