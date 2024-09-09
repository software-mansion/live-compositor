import LiveCompositor from '@live-compositor/node';
import { InputStream, Tiles, Rescaler, View, useInputStreams, Image } from 'live-compositor';
import { downloadAllAssets, sleep } from './utils';
import { useState, useEffect } from 'react';
import path from 'path';
import fs from 'fs-extra';

function App() {
  const [startTransition, setStartTransition] = useState(false);
  const [showVideo, setShowVideo] = useState(false);
  useEffect(() => {
    if (startTransition && showVideo) {
      return;
    }
    const timeout = setTimeout(
      () => {
        if (!startTransition) {
          setStartTransition(true);
        } else {
          setShowVideo(true);
        }
      },
      startTransition ? 500 : 4000
    );
    return () => {
      clearTimeout(timeout);
    };
  }, [showVideo, startTransition]);
  return (
    <View>
      <Rescaler top={0} left={0}>
        <Image imageId="background" />
      </Rescaler>
      {showVideo ? (
        <View>
          <Rescaler bottom={20} right={20} width={1120} height={630}>
            <Scene />
          </Rescaler>
          <Rescaler top={20} left={20} width={992} height={380}>
            <Image imageId="code" />
          </Rescaler>
        </View>
      ) : startTransition ? (
        <Rescaler top={20} left={20} width={992} height={380} transition={{ durationMs: 500 }}>
          <Image imageId="code" />
        </Rescaler>
      ) : (
        <Rescaler top={50} left={50} width={1820} height={980}>
          <Image imageId="code" />
        </Rescaler>
      )}
    </View>
  );
}

function Scene() {
  const inputs = useInputStreams();
  return (
    <Tiles transition={{ durationMs: 400 }} backgroundColor="#000000FF" width={1920} height={1080}>
      {Object.values(inputs).map(({ inputId }) => (
        <InputStream key={inputId} inputId={inputId} />
      ))}
    </Tiles>
  );
}

//function InputTile({ inputId }: { inputId: string }) {
//  return (
//    <View>
//      <Rescaler mode="fill">
//        <InputStream inputId={inputId} />
//      </Rescaler>
//      <View bottom={0} left={0} height={50} backgroundColor="#FFFFFF88">
//        <Text fontSize={40} color="#FF0000">
//          {' '}
//          Input ID: {inputId}
//        </Text>
//      </View>
//    </View>
//  );
//}

async function run() {
  await fs.mkdirp(path.join(__dirname, '../.workingdir'));
  await downloadAllAssets();
  const compositor = await LiveCompositor.create();

  const RESOLUTION = {
    width: 1920,
    height: 1080,
  } as const;
  const VIDEO_ENCODER_OPTS = {
    type: 'ffmpeg_h264',
    preset: 'veryfast',
  } as const;

  await compositor.registerImage('code', {
    asset_type: 'png',
    path: path.join(__dirname, '../code.png'),
  });

  await compositor.registerImage('background', {
    asset_type: 'png',
    path: path.join(__dirname, '../background.png'),
  });

  await compositor.registerImage('spinner', {
    asset_type: 'gif',
    path: path.join(__dirname, './buffering.gif'),
  });

  await compositor.registerOutput('output', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.workingdir/rtcon2.mp4'),
    video: {
      encoder: VIDEO_ENCODER_OPTS,
      resolution: RESOLUTION,
      root: <App />,
    },
    audio: {
      encoder: {
        type: 'aac',
        channels: 'stereo',
      },
    },
  });

  await compositor.registerInput('input_1', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.assets/BigBuckBunny.mp4'),
  });
  await compositor.start();

  await sleep(7_000);
  await compositor.registerInput('input_2', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.assets/Sintel2.mp4'),
  });

  await sleep(4_000);
  await compositor.registerInput('input_3', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.assets/ElephantsDream.mp4'),
  });
  // using stream instead of mp4 so there is delay between registration and starting stream
  // await compositor.registerInput('input_2', {
  //   type: 'rtp_stream',
  //   port: 8003,
  //   video: {
  //     decoder: 'ffmpeg_h264',
  //   },
  // });
  // ffmpegSendVideoFromMp4(8003, path.join(__dirname, '../.assets/ElephantsDream.mp4'));

  await sleep(10_000);
  await compositor.unregisterOutput('output');
}
run();
