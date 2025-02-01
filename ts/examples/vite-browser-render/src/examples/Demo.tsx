import { useEffect, useRef, useState } from 'react';
import { LiveCompositor } from '@live-compositor/web-wasm';
import { InputStream, Mp4, Rescaler, Text, useInputStreams, View } from 'live-compositor';

function DemoExample() {
  const [compositor, setCompositor] = useState<LiveCompositor | undefined>();
  const previewRef = useRef<HTMLVideoElement>(null);
  const [hasCamera, setCamera] = useState<boolean>();
  const [hasScreenCapture, setScreenCapture] = useState<boolean>();

  useEffect(() => {
    const compositor = new LiveCompositor({});
    let terminate = false;

    const startPromise = (async () => {
      await compositor.init();
      if (!terminate) {
        setCompositor(compositor);
      }
    })();

    return () => {
      terminate = true;
      void (async () => {
        await startPromise;
        await compositor.terminate();
      })();
    };
  }, []);

  useEffect(() => {
    if (!compositor) {
      return;
    }

    void (async () => {
      const queryParams = new URLSearchParams(window.location.search);
      const streamKey = queryParams.get('twitchKey');
      if (!streamKey) {
        throw new Error('Add "twitchKey" query params with your Twitch stream key.');
      }

      const { stream } = await compositor.registerOutput('output', <Scene />, {
        type: 'whip',
        endpointUrl: 'https://g.webrtc.live-video.net:4443/v2/offer',
        bearerToken: streamKey,
        video: {
          resolution: { width: 1920, height: 1080 },
          maxBitrate: 6_000_000,
        },
        audio: true,
      });

      await compositor.start();

      if (stream && previewRef.current) {
        previewRef.current.srcObject = stream;
        await previewRef.current.play();
      }
    })();
  }, [compositor]);

  const toggleCamera = async () => {
    try {
      setCamera(!hasCamera);
      if (hasCamera) {
        await compositor?.unregisterInput('camera');
      } else {
        await compositor?.registerInput('camera', { type: 'camera' });
      }
    } catch (err) {
      console.warn(err, 'Failed to capture camera output');
    }
  };

  const toggleShareScreen = async () => {
    try {
      setScreenCapture(!hasScreenCapture);
      if (hasScreenCapture) {
        await compositor?.unregisterInput('screen');
      } else {
        await compositor?.registerInput('screen', { type: 'screen_capture' });
      }
    } catch (err) {
      console.warn(err, 'Failed to capture screen output');
    }
  };

  return (
    <div className="card">
      <div style={{ display: 'flex', flexDirection: 'row' }}>
        <button style={{ margin: 10 }} onClick={toggleCamera}>
          Toggle camera
        </button>
        <button style={{ margin: 10 }} onClick={toggleShareScreen}>
          Toggle share screen
        </button>
      </div>
      <video ref={previewRef} style={{ width: 1280, height: 720 }} />
    </div>
  );
}

const MP4_URL =
  'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4';

function Scene() {
  const inputs = useInputStreams();

  const hasCamera = !!inputs.camera;
  const hasScreenCapture = !!inputs.screen;

  return (
    <View>
      {hasScreenCapture ? (
        <>
          <Rescaler style={{ rescaleMode: 'fill' }}>
            <InputStream inputId="screen" />
          </Rescaler>
          <View style={{ top: 40, left: 40, width: 480, height: 270, backgroundColor: 'black' }}>
            <Rescaler>
              <Mp4 source={MP4_URL} />
            </Rescaler>
          </View>
        </>
      ) : (
        <>
          <Rescaler style={{ rescaleMode: 'fill' }}>
            <Mp4 source={MP4_URL} />
          </Rescaler>
          <View
            style={{
              direction: 'column',
              left: 40,
              top: 40,
              width: 480,
              height: 270,
              padding: 20,
              backgroundColor: 'black',
            }}>
            <View />
            <Text
              style={{
                align: 'center',
                backgroundColor: 'red',
                fontSize: 40,
                maxWidth: 440,
              }}>
              Unable to share a screen
            </Text>
            <View />
          </View>
        </>
      )}
      {hasCamera ? (
        <Rescaler style={{ right: 40, top: 40, width: 480, height: 270 }}>
          <InputStream inputId="camera" />
        </Rescaler>
      ) : (
        <View
          style={{
            direction: 'column',
            right: 40,
            top: 40,
            width: 480,
            height: 270,
            backgroundColor: 'black',
            padding: 20,
          }}>
          <View />
          <Text
            style={{
              align: 'center',
              backgroundColor: 'red',
              fontSize: 40,
              maxWidth: 440,
            }}>
            Unable to access camera
          </Text>
          <View />
        </View>
      )}
      <View style={{ height: 40, backgroundColor: '#000000', bottom: 0, left: 0, padding: 10 }}>
        <Text style={{ fontSize: 40, fontFamily: 'Noto Sans' }}>
          Demo: stream your camera and screen to Twitch
        </Text>
      </View>
    </View>
  );
}

export default DemoExample;
