import LiveCompositor from '@live-compositor/node';
import { View, Text } from 'live-compositor';
import { ffplayStartPlayerAsync } from './liveCompositorFfplayHelper';

function App() {
  return (
    <View style={{ direction: 'column' }}>
      <View />
      <Text style={{ fontSize: 50 }}>Open index.ts and get started</Text>
      <View style={{ height: 20 }} />
      <Text style={{ width: 1000, fontSize: 30, wrap: 'word' }}>
        This example renders static text and sends the output stream via RTP to local port 8001.
        Generated code includes helpers in liveCompositorFfplayHelper.ts that display the output
        stream using ffplay, make sure to remove them for any real production use.
      </Text>
      <View />
    </View>
  );
}

async function run() {
  const compositor = new LiveCompositor();
  await compositor.init();

  // Display output with `ffplay`.
  await ffplayStartPlayerAsync('127.0.0.0', 8001);

  await compositor.registerOutput('output_1', {
    type: 'rtp_stream',
    port: 8001,
    ip: '127.0.0.1',
    transportProtocol: 'udp',
    video: {
      encoder: {
        type: 'ffmpeg_h264',
        preset: 'ultrafast',
      },
      resolution: {
        width: 1920,
        height: 1080,
      },
      root: <App />,
    },
  });

  // Connect any additional inputs/images/shader you might need before the start.

  await compositor.start();
}
void run();
