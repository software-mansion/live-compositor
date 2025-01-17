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
  const compositor = new LiveCompositor();
  await compositor.init();

  // register input/outputs/images/shaders/...

  await compositor.registerOutput('example_output', {
    type: 'rtp_stream',
    port: 8001,
    ip: '127.0.0.1',
    transportProtocol: 'udp',
    video: {
      encoder: { type: 'ffmpeg_h264', preset: 'ultrafast' },
      resolution: { width: 1920, height: 1080 },
      root: <ExampleApp />,
    },
    audio: {
      encoder: {
        type: 'opus',
        channels: 'stereo',
      },
    },
  });

  await compositor.start();
}
run();
```

See our [docs](https://compositor.live/docs) to learn more.

## License

`@live-compositor/node` is MIT licensed, but internally it is downloading and using Live Compositor server that is licensed
under [Business Source License 1.1](https://github.com/software-mansion/live-compositor/blob/master/LICENSE).

## LiveCompositor is created by Software Mansion

[![swm](https://logo.swmansion.com/logo?color=white&variant=desktop&width=150&tag=live-compositor-github 'Software Mansion')](https://swmansion.com)

Since 2012 [Software Mansion](https://swmansion.com) is a software agency with experience in building web and mobile apps as well as complex multimedia solutions. We are Core React Native Contributors and experts in live streaming and broadcasting technologies. We can help you build your next dream product – [Hire us](https://swmansion.com/contact/projects?utm_source=live-compositor&utm_medium=readme).
