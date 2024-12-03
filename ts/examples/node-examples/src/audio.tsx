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
      <InputTile inputId="input_1" muted={streamWithAudio === 'input_1'} />
      <InputTile inputId="input_2" muted={streamWithAudio === 'input_2'} />
    </Tiles>
  );
}

function InputTile({ inputId, muted }: { inputId: string; muted: boolean }) {
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
        <InputStream inputId={inputId} volume={volume} muted={muted} />
      </Rescaler>
      <View style={{ bottom: 10, left: 10, height: 40 }}>
        <Text style={{ fontSize: 40 }}>
          Input ID: {inputId}, volume: {volume.toFixed(2)} {muted ? 'muted' : 'live'}
        </Text>
      </View>
    </View>
  );
}

async function run() {
  await downloadAllAssets();
  const compositor = new LiveCompositor();
  await compositor.init();

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
    },
  });
  void gstReceiveTcpStream('127.0.0.1', 8001);

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
void run();
