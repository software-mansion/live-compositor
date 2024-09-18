import { RefObject, useEffect, useRef, useState } from 'react';
import { LiveCompositor } from '@live-compositor/web';
import { InputStream, Text, useInputStreams, View } from 'live-compositor';

const BUNNY_URL = 'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4';

function MP4Player() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const compositor = useCompositor(canvasRef);

  useEffect(() => {
    if (compositor == null) {
      return;
    }

    const stopQueue = compositor.start();
    return () => stopQueue();
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
  const inputState = inputs['bunny_video'].videoState;

  if (inputState == 'ready') {
    return (
      <View backgroundColor="#000000">
        <View width={530} height={40} bottom={300} left={500}>
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

function useCompositor(canvasRef: RefObject<HTMLCanvasElement>): LiveCompositor | undefined {
  const [compositor, setCompositor] = useState<LiveCompositor | undefined>(undefined);
  useEffect(() => {
    if (!canvasRef.current) {
      return;
    }

    // TODO(noituri): Fix issue with compositor starting multiple times
    const setupCompositor = async () => {
      const compositor = await LiveCompositor.create({
        framerate: {
          num: 30,
          den: 1,
        },
        streamFallbackTimeoutMs: 500,
      });

      setCompositor(compositor);

      await compositor.registerFont(
        'https://fonts.gstatic.com/s/notosans/v36/o-0mIpQlx3QUlC5A4PNB6Ryti20_6n1iPHjcz6L1SoM-jCpoiyD9A-9a6Vc.ttf'
      );
      await compositor.registerInput('bunny_video', { type: 'mp4', url: BUNNY_URL });
      await compositor.registerOutput('output', {
        type: 'canvas',
        canvas: canvasRef.current!,
        resolution: {
          width: 1280,
          height: 720,
        },
        root: <Scene />,
      });
    }

    setupCompositor();
  }, [canvasRef]);

  return compositor;
}

export default MP4Player;
