import { gstStartWebcamStream } from '../utils/gst';
import { downloadAsync } from '../utils/utils';
import { ffmpegSendVideoFromMp4, ffplayStartPlayerAsync } from '../utils/ffmpeg';
import path from 'path';
import LiveCompositor from '@live-compositor/node';
import { Rescaler, Text, Image, View, InputStream, Tiles } from 'live-compositor';
import { useEffect, useState } from 'react';

const OUTPUT_RESOLUTION = {
  width: 1920,
  height: 1080,
};

const INPUT_PORT = 8002;
const OUTPUT_PORT = 8004;

const BACKGROUND_URL =
  'https://raw.githubusercontent.com/membraneframework-labs/video_compositor_snapshot_tests/main/demo_assets/triangles_background.png';
const CALL_URL =
  'https://raw.githubusercontent.com/membraneframework-labs/video_compositor_snapshot_tests/main/demo_assets/call.mp4';

/**
 * Example is switching between following number of tiles
 */
const inputCountPhases = [
  2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1,
];

function App() {
  const [counter, setCounter] = useState(0);
  useEffect(() => {
    const timeout = setTimeout(() => {
      setCounter(counter + 1);
    }, 2000);
    return () => clearTimeout(timeout);
  }, [counter]);

  return <CallWithMockedInputs inputCount={inputCountPhases[counter % inputCountPhases.length]} />;
}

function CallWithMockedInputs({ inputCount }: { inputCount: number }) {
  const tiles = [...Array(inputCount)].map((_value, index) => (
    <VideoCallTile key={index} id={index} />
  ));
  return (
    <View>
      <Rescaler top={0} left={0}>
        <Image imageId="background" />
      </Rescaler>
      <Tiles
        padding={5}
        transition={{
          durationMs: 700,
          easingFunction: {
            functionName: 'cubic_bezier',
            points: [0.35, 0.22, 0.1, 0.8],
          },
        }}>
        {tiles}
      </Tiles>
    </View>
  );
}

function VideoCallTile({ id }: { id: number }) {
  return (
    <View>
      <Rescaler>
        <InputStream inputId="input_1" />
      </Rescaler>
      <View height={50} bottom={0} left={0}>
        <View />
        <Text
          fontSize={25}
          align="center"
          color="#FFFFFF"
          backgroundColor="#FF0000"
          fontFamily="Arial">
          InputStream {id} 🚀
        </Text>
        <View />
      </View>
    </View>
  );
}

async function exampleAsync() {
  const useWebCam = process.env.SMELTER_WEBCAM !== 'false';
  const compositor = new LiveCompositor();
  await compositor.init();

  await compositor.registerImage('background', {
    assetType: 'png',
    url: BACKGROUND_URL,
  });

  await compositor.registerInput('input_1', {
    type: 'rtp_stream',
    transportProtocol: useWebCam ? 'tcp_server' : 'udp',
    port: INPUT_PORT,
    video: {
      decoder: 'ffmpeg_h264',
    },
  });

  await ffplayStartPlayerAsync(OUTPUT_PORT);
  await compositor.registerOutput('output_1', {
    type: 'rtp_stream',
    ip: '127.0.0.1',
    port: OUTPUT_PORT,
    video: {
      resolution: OUTPUT_RESOLUTION,
      encoder: {
        type: 'ffmpeg_h264',
        preset: 'ultrafast',
      },
      root: <App />,
    },
  });

  if (useWebCam) {
    await gstStartWebcamStream(INPUT_PORT);
  } else {
    const callPath = path.join(__dirname, '../assets/call.mp4');
    await downloadAsync(CALL_URL, callPath);
    void ffmpegSendVideoFromMp4(INPUT_PORT, callPath);
  }
  await compositor.start();
}

void exampleAsync();
