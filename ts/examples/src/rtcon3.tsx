import LiveCompositor from '@live-compositor/node';
import { Text, InputStream, Tiles, Rescaler, View, useInputStreams, Image } from 'live-compositor';
import { downloadAllAssets, sleep } from './utils';
import path from 'path';
import fs from 'fs-extra';
import { useState, useEffect } from 'react';

function App() {
  const inputs = useInputStreams();
  return (
    <View>
      <Tiles transition={{ durationMs: 400 }}>
        {Object.values(inputs).map(({ inputId, videoState }) =>
          videoState === 'playing' ? (
            <InputTile key={inputId} inputId={inputId} />
          ) : (
            <Image key={inputId} imageId="spinner" />
          )
        )}
      </Tiles>
      <NewInputToast />
    </View>
  );
}

function NewInputToast() {
  const inputs = useInputStreams();
  const [current, setCurrent] = useState<string | null>(null);
  const [prevStreams, setPrevStreams] = useState<string[]>([]);

  useEffect(() => {
    const currentStreams = Object.keys(inputs);
    const newStream = currentStreams.find(stream => !prevStreams.includes(stream));
    setPrevStreams(currentStreams);
    if (newStream) {
      setCurrent(newStream);
    }
  }, [inputs]);

  useEffect(() => {
    if (!setCurrent) {
      return;
    }
    const timeout = setTimeout(() => {
      setCurrent(null);
    }, 3000);
    return () => clearTimeout(timeout);
  }, [current]);

  return current ? (
    <View top={0} left={0} backgroundColor="#FFFFFFFF" height={50}>
      <Text fontSize={40} color="#FF0000">
        Connecting new stream: {current}
      </Text>
    </View>
  ) : undefined;
}

function InputTile({ inputId }: { inputId: string }) {
  return (
    <View>
      <Rescaler mode="fill">
        <InputStream inputId={inputId} />
      </Rescaler>
      <View bottom={0} left={0} height={50} backgroundColor="#FFFFFF88">
        <Text fontSize={40} color="#FF0000">
          {' '}
          Input ID: {inputId}
        </Text>
      </View>
    </View>
  );
}

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
    preset: 'ultrafast',
  } as const;

  await compositor.registerImage('spinner', {
    asset_type: 'gif',
    path: path.join(__dirname, './buffering.gif'),
  });

  await compositor.registerOutput('output', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.workingdir/rtcon3.mp4'),
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

  await sleep(4_000);
  await compositor.registerInput('input_2', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.assets/ElephantsDream.mp4'),
  });

  await sleep(10_000);
  await compositor.unregisterOutput('output');
  await sleep(1_000);
  process.exit(0);
}
run();
