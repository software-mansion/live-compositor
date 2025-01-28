import { useEffect, useState } from 'react';
import { LiveCompositor } from '@live-compositor/web-wasm';
import { Mp4, Rescaler, Text, View } from 'live-compositor';

function WhipExample() {
  const [compositor, setCompositor] = useState<LiveCompositor | undefined>();

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
      await compositor.registerOutput('output', <Scene />, {
        type: 'whip',
        endpointUrl: 'https://g.webrtc.live-video.net:4443/v2/offer',
        bearerToken: streamKey,
        video: {
          resolution: { width: 1920, height: 1080 },
          maxBitrate: 2_000_000,
        },
      });

      await compositor.start();
    })();
  }, [compositor]);

  return <div className="card">Streaming</div>;
}

const MP4_URL =
  'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4';

function Scene() {
  return (
    <View>
      <Rescaler>
        <Mp4 source={MP4_URL} />
      </Rescaler>
      <View style={{ width: 300, height: 40, backgroundColor: '#000000', bottom: 100, left: 520 }}>
        <Text style={{ fontSize: 30, fontFamily: 'Noto Sans' }}>WHIP example</Text>
      </View>
    </View>
  );
}

export default WhipExample;
