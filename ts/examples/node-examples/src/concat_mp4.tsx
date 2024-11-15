import { OfflineCompositor } from '@live-compositor/node';
import { View, Text, Rescaler, InputStream, Slides, Slide } from 'live-compositor';
import { downloadAllAssets } from './utils';
import path from 'path';
import { useTimeLimitedComponent } from '../../../live-compositor/cjs/context/childrenLifetimeContext';

function ExampleApp() {
  return (
    <View>
      <Slides>
        <Slide>
          <Input inputId="input_1" endTimestamp={3_000} />
        </Slide>
        <Slide>
          <Input inputId="input_2" endTimestamp={6_000} />
        </Slide>
        <Slide durationMs={3_000}>
          <Input inputId="input_1" endTimestamp={10_000} />
        </Slide>
      </Slides>
    </View>
  );
}

function Input({ inputId, endTimestamp }: { inputId: string; endTimestamp: number }) {
  // Temporary, useTimeLimitedComponent is an internal hook, InputStream component will rely
  // on the mp4 length returned from the compositor
  useTimeLimitedComponent(endTimestamp);
  return (
    <View>
      <Rescaler>
        <InputStream inputId={inputId} />
      </Rescaler>
      <View style={{ bottom: 10, left: 10, height: 50 }}>
        <Text
          style={{ fontSize: 40, color: '#FF0000', lineHeight: 50, backgroundColor: '#FFFFFF88' }}>
          Input ID: {inputId}
        </Text>
      </View>
    </View>
  );
}

async function run() {
  await downloadAllAssets();
  const compositor = new OfflineCompositor();
  await compositor.init();

  await compositor.registerInput('input_1', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.assets/BigBuckBunny.mp4'),
    offsetMs: 0,
    required: true,
  });

  await compositor.registerInput('input_2', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.assets/ElephantsDream.mp4'),
    offsetMs: 5000,
    required: true,
  });

  await compositor.render(
    {
      type: 'mp4',
      serverPath: path.join(__dirname, '../.assets/concat_mp4_output.mp4'),
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
    },
    10000
  );
  process.exit(0);
}
void run();
