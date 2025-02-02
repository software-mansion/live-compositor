import path from 'path';
import * as readline from 'readline';

import { ffmpegSendVideoFromMp4 } from '../utils/ffmpeg';
import { downloadAsync } from '../utils/utils';
import { gstStartPlayer, gstStartWebcamStream } from '../utils/gst';
import LiveCompositor from '@live-compositor/node';
import { InputStream, Text, Image, Rescaler, View } from 'live-compositor';
import { useEffect, useState } from 'react';

const OUTPUT_RESOLUTION = {
  width: 1920,
  height: 1080,
};

const WEBCAM_INPUT_PORT = 10000;
const OUTPUT_PORT = 10004;

const GAMEPLAY_URL =
  'https://raw.githubusercontent.com/membraneframework-labs/video_compositor_snapshot_tests/main/demo_assets/gameplay.mp4';
const DONATE_URL =
  'https://raw.githubusercontent.com/membraneframework-labs/video_compositor_snapshot_tests/main/demo_assets/donate.gif';
const CALL_URL =
  'https://raw.githubusercontent.com/membraneframework-labs/video_compositor_snapshot_tests/main/demo_assets/call.mp4';

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

function App() {
  return (
    <View>
      <Rescaler>
        <InputStream inputId="gameplay" mute />
      </Rescaler>
      <Rescaler
        width={500}
        height={300}
        top={30}
        right={30}
        verticalAlign="top"
        horizontalAlign="right">
        <InputStream inputId="webcam_input" />
      </Rescaler>
      <Donates />
    </View>
  );
}

function Donates() {
  const [message, setMessage] = useState<string>('init');
  const [show, setShow] = useState<boolean>(false);

  useEffect(() => {
    if (!show) {
      // show new terminal prompt if previous donation just finished
      rl.question('Enter donate content: ', message => {
        setMessage(message);
        setShow(true);
      });
      return;
    } else {
      const timeout = setTimeout(() => {
        setShow(false);
      }, 2000);
      return () => clearTimeout(timeout);
    }
  }, [show]);

  return <DonateCard message={message} show={show} />;
}

function DonateCard({ message, show }: { message: string; show: boolean }) {
  const width = 480;
  const top = show ? 30 : -330;

  return (
    <View
      top={top}
      left={0}
      height={330}
      direction="row"
      transition={{
        durationMs: 1000,
        easingFunction: 'bounce',
      }}>
      <View />
      <View direction="column" width={width}>
        <Image imageId="donate" />
        <Text
          weight="extra_bold"
          width={width}
          fontSize={50}
          align="center"
          color="#FF0000"
          fontFamily="Comic Sans MS">
          {message}
        </Text>
      </View>
      <View />
    </View>
  );
}

async function exampleAsync() {
  const compositor = new LiveCompositor();
  await compositor.init();

  const gameplayPath = path.join(__dirname, '../assets/gameplay.mp4');
  await downloadAsync(GAMEPLAY_URL, gameplayPath);

  await compositor.registerImage('donate', {
    assetType: 'gif',
    url: DONATE_URL,
  });

  const useWebCam = process.env.SMELTER_WEBCAM !== 'false';
  await compositor.registerInput('webcam_input', {
    type: 'rtp_stream',
    port: WEBCAM_INPUT_PORT,
    transportProtocol: useWebCam ? 'tcp_server' : 'udp',
    video: {
      decoder: 'ffmpeg_h264',
    },
  });

  await compositor.registerInput('gameplay', {
    type: 'mp4',
    serverPath: gameplayPath,
  });

  await compositor.registerOutput('video_output', {
    type: 'rtp_stream',
    port: OUTPUT_PORT,
    transportProtocol: 'tcp_server',
    video: {
      resolution: OUTPUT_RESOLUTION,
      encoder: {
        type: 'ffmpeg_h264',
        preset: 'ultrafast',
      },
      root: <App />,
    },
    audio: {
      encoder: {
        type: 'opus',
        channels: 'stereo',
      },
    },
  });
  gstStartPlayer(OUTPUT_PORT);

  if (useWebCam) {
    await gstStartWebcamStream(WEBCAM_INPUT_PORT);
  } else {
    const callPath = path.join(__dirname, '../assets/call.mp4');
    await downloadAsync(CALL_URL, callPath);
    void ffmpegSendVideoFromMp4(WEBCAM_INPUT_PORT, callPath);
  }

  await compositor.start();
}

void exampleAsync();
