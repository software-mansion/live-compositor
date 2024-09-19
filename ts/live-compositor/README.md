# `live-compositor`

This package provides a set of React components that can be used to define a composition of a video stream. Available components can only be used with React renderers specific for Live Compositor. Currently, we only support Node.js runtime with `@live-compositor/node` package, but support for more environments is planned in the future.

Live Components should not be mixed with other React or React Native components, but you can still use hooks like `useState` from React.

## Usage

```tsx
import LiveCompositor from '@live-compositor/node';
import { View, Text, InputStream, Rescaler } from 'live-compositor';

function ExampleApp() {
  return (
    <View direction="column">
      <Rescaler mode="fill">
        <InputStream inputId="example_input_1" />
      </Rescaler>
      <Text fontSize={20}>Example label</Text>
    </View>
  );
}

async function run() {
  const compositor = new LiveCompositor();
  await compositor.init()

  await compositor.registerInput('example_input_1', {
    ...EXAMPLE_INPUT_OPTIONS,
  })
  await compositor.registerOutput('example_output', {
    ...OUTPUT_STREAM_OPTIONS,
    video: {
      ...OUTPUT_VIDEO_OPTIONS
      root: <ExampleApp />,
    },
  });
  await compositor.start();
}
run();
```

## Documentation

Components props are documented via TypeScript types definitions in this package. You can also refer to LiveCompositor (e.g. [View](https://compositor.live/docs/api/components/View)), but keep in mind to convert field names from snake case to camel case.

See https://compositor.live/docs
