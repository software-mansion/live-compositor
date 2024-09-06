# `@live-compositor/node`

Provides API to create and manage LiveCompositor instances for Node.js environment.

When you call `registerOutput` on the LiveCompositor instance, you can pass a `ReactElement` that represents a component tree built from components included in `live-compositor` package. Those components will define what will be rendered on the output stream.

## Usage

```tsx
import LiveCompositor from '@live-compositor/node';
import { View, Text } from 'live-compositor';

function ExampleApp() {
  return (
    <View>
      <Text fontSize={20}>Hello world</Text>
    </View>
  );
}

async function run() {
  const compositor = await LiveCompositor.create();

  // register input/outputs/images/shaders/...

  await compositor.registerOutput('example_output', {
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
}
run();
```
