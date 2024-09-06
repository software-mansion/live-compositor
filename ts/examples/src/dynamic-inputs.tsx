import LiveCompositor from '@live-compositor/node';
import { useInputStreams, Text, InputStream, Tiles, Rescaler, View } from 'live-compositor';
import { downloadAllAssets, ffplayStartPlayerAsync, sleep } from './utils';
import path from 'path';

function ExampleApp() {
  const inputs = useInputStreams();
  return (
    <Tiles transition={{ durationMs: 200 }}>
      {Object.values(inputs).map(input =>
        !input.videoState ? (
          <Text key={input.inputId} fontSize={40}>
            Waiting for stream {input.inputId} to connect
          </Text>
        ) : input.videoState === 'playing' ? (
          <InputTile key={input.inputId} inputId={input.inputId} />
        ) : input.videoState === 'finished' ? (
          <Text key={input.inputId} fontSize={40}>
            Stream {input.inputId} finished
          </Text>
        ) : (
          'Fallback'
        )
      )}
    </Tiles>
  );
}

function InputTile({ inputId }: { inputId: string }) {
  return (
    <View>
      <Rescaler>
        <InputStream inputId={inputId} />
      </Rescaler>
      <View bottom={10} left={10} height={40}>
        <Text fontSize={40}>Input ID: {inputId}</Text>
      </View>
    </View>
  );
}

async function run() {
  await downloadAllAssets();
  const compositor = await LiveCompositor.create();

  ffplayStartPlayerAsync('127.0.0.1', 8001);
  await sleep(2000);

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
      root: <ExampleApp />,
    },
  });
  await compositor.start();

  await sleep(5000);
  await compositor.registerInput('input_1', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.assets/BigBuckBunny.mp4'),
  });

  await sleep(5000);
  await compositor.registerInput('input_2', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.assets/ElephantsDream.mp4'),
  });
}
run();
