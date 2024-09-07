import LiveCompositor from '@live-compositor/node';
import { Text, InputStream, Rescaler, View } from 'live-compositor';
import { downloadAllAssets, sleep } from './utils';
import path from 'path';
import fs from 'fs-extra';

//
// Ok, but how you actually define how video should be composed. Currently,
// API looks like this
//
// <show json that represent rtcon1 example>
// and results in video like this
// <show video rtcon1>
//
// Scene is built from components.
// <point out few elements of the scene e.g. 'text' just renders text, 'rescaler' takes it it's child and resizes it>
//
// As you can see this structure has very similar structure as HTML. We are currently
// working on implementing React support for this API. Instead of json you can write code
// like this
//
// <show rtcon1 example>
// <talk about how components map to json> (I wouldn't show video again at this point)
//
// Ok, so let's extract this scene to the component
// <show InputTile from rtcon2>
//
// And show all the connected input streams in Tile component
//
// <show App from rtcon2>
//
// Using `useInputStreams` hook we get information what streams are connected and can update
// when something changes
//
// <show video rtcon2>
//
// In this example we start with one stream and second stream connects after few seconds
//

function App() {
  return (
    <View>
      <Rescaler mode="fill">
        <InputStream inputId="input_1" />
      </Rescaler>
      <View bottom={0} left={0} height={50} backgroundColor="#FFFFFF88">
        <Text fontSize={40} color="#FF0000">
          Input ID: input_1
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

  await compositor.registerOutput('output', {
    type: 'mp4',
    serverPath: path.join(__dirname, '../.workingdir/rtcon1.mp4'),
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

  await sleep(10_000);
  await compositor.unregisterOutput('output');
  await sleep(1_000);
  process.exit(0);
}
run();
