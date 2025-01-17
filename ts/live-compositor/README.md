# `live-compositor`

This package provides a set of React components that can be used to define a composition of a video stream. Available components can only be used with React renderers specific for Live Compositor. Currently, we only support Node.js runtime with `@live-compositor/node` package, but support for more runtimes is planned in the future.

Live Compositor components should not be mixed with other React or React Native components, but you can still use hooks like `useState`/`useEffect` from React.

## Getting started

To try compositor generate started project by running:

```
npm create live-compositor
```

## Usage

```tsx
import { View, Text, InputStream, Rescaler } from 'live-compositor';

function ExampleApp() {
  return (
    <View style={{ direction: 'column' }}>
      <Rescaler style={{ rescaleMode: 'fill' }}>
        <InputStream inputId="example_input_1" />
      </Rescaler>
      <Text fontSize={20}>Example label</Text>
    </View>
  );
}
```

Check out [@live-compositor/node](https://www.npmjs.com/package/@live-compositor/node) to learn how to use those components for video composition.

See our [docs](https://compositor.live/docs) to learn more.

## License

`live-compositor` package is MIT licensed, but it is only useful when used with Live Compositor server that is licensed
under [Business Source License 1.1](https://github.com/software-mansion/live-compositor/blob/master/LICENSE).

## LiveCompositor is created by Software Mansion

[![swm](https://logo.swmansion.com/logo?color=white&variant=desktop&width=150&tag=live-compositor-github 'Software Mansion')](https://swmansion.com)

Since 2012 [Software Mansion](https://swmansion.com) is a software agency with experience in building web and mobile apps as well as complex multimedia solutions. We are Core React Native Contributors and experts in live streaming and broadcasting technologies. We can help you build your next dream product – [Hire us](https://swmansion.com/contact/projects?utm_source=live-compositor&utm_medium=readme).
