import { OfflineCompositor } from '@live-compositor/node';
import { View, Text, Rescaler, InputStream, SlideShow, Slide } from 'live-compositor';
import { downloadAllAssets } from './utils';
import path from 'path';

function ExampleApp() {
  return (
    <View>
      <SlideShow>
        <Slide>
          <Input inputId="input_1" />
        </Slide>
        <Slide durationMs={3_000}>
          <View>
            <Text
              style={{
                fontSize: 40,
                color: '#FF0000',
                lineHeight: 50,
                backgroundColor: '#FFFFFF88',
              }}>
              Input
            </Text>
          </View>
        </Slide>
        <Slide>
          <Input inputId="input_2" />
        </Slide>
        <Slide durationMs={3_000}>
          <View>
            <Text
              style={{
                fontSize: 40,
                color: '#FF0000',
                lineHeight: 50,
                backgroundColor: '#FFFFFF88',
              }}>
              Input
            </Text>
          </View>
        </Slide>
      </SlideShow>
    </View>
  );
}

function Input({ inputId }: { inputId: string }) {
  return (
    <View>
      <Rescaler>
        <InputStream inputId={inputId} volume={1.0} />
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
    //serverPath: path.join(__dirname, '../.assets/BigBuckBunny.mp4'),
    serverPath: '/home/wojtek/Downloads/sd.mp4',
    offsetMs: 0,
    required: true,
  });

  await compositor.registerInput('input_2', {
    type: 'mp4',
    serverPath: '/home/wojtek/Downloads/sd_no_audio.mp4',
    offsetMs: 10_000,
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
      audio: {
        encoder: {
          type: 'aac',
          channels: 'stereo',
        },
      },
    },
    80000
  );
  process.exit(0);
}
void run();
