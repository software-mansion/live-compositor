import LiveCompositor from '@live-compositor/node';
import { Text, InputStream, Tiles, Rescaler, View } from 'live-compositor';
import { downloadAllAssets, gstReceiveTcpStream, sleep } from './utils';
import path from 'path';
import { useState, useEffect } from 'react';

function ExampleApp() {
  const [streamWithAudio, setStream] = useState('input_1');
  useEffect(() => {
    const timeout = setTimeout(() => {
      setStream(streamWithAudio === 'input_1' ? 'input_2' : 'input_1');
    }, 5000);
    return () => clearTimeout(timeout);
  }, [streamWithAudio]);

  return (
    <Tiles transition={{ durationMs: 200 }}>
      <InputTile inputId="input_1" mute={streamWithAudio === 'input_1'} />
      <InputTile inputId="input_2" mute={streamWithAudio === 'input_2'} />
    </Tiles>
  );
}

function InputTile({ inputId, mute }: { inputId: string; mute: boolean }) {
  const [volume, setVolume] = useState(1.0);

  useEffect(() => {
    const timeout = setTimeout(() => {
      if (volume < 0.2) {
        setVolume(1.0);
      } else {
        setVolume(volume - 0.1);
      }
    }, 1000);
    return () => clearTimeout(timeout);
  }, [volume]);

  return (
    <View>
      <Rescaler>
        <InputStream inputId={inputId} volume={volume} mute={mute} />
      </Rescaler>
      <View bottom={10} left={10} height={40}>
        <Text fontSize={40}>
          Input ID: {inputId}, volume: {volume.toFixed(2)} {mute ? 'muted' : 'live'}
        </Text>
      </View>
    </View>
  );
}

async function run() {
  await downloadAllAssets();
  const compositor = await LiveCompositor.create();

  await sleep(2000);

  await compositor.registerOutput('output_1', {
    type: 'rtp_stream',
    port: 8001,
    transportProtocol: 'tcp_server',
    video: {
      encoder: {
        type: 'ffmpeg_h264',
        preset: 'ultrafast',
      },
      resolution: {
        width: 1920,
        height: 1080,
      },
      root: <ExampleApp />,
    },
    audio: {
      encoder: {
        type: 'opus',
        channels: 'stereo',
      },
      initial: { inputs: [] },
    },
  });
  gstReceiveTcpStream('127.0.0.1', 8001);

  await compositor.registerInput('input_1', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.assets/BigBuckBunny.mp4'),
  });

  await compositor.registerInput('input_2', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.assets/ElephantsDream.mp4'),
  });

  await compositor.start();
}
run();
