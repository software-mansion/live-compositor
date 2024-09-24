# TypeScript SDK Reference

## `new LiveCompositor()`

Packages like `@live-compositor/node` export `LiveCompositor` class that is a main entity used to interact with or control Live Compositor server instance.

See [LiveCompositor API](./instance.md).

## Components

React components that can be used to define how input streams should be composed.

- [`InputStream`](./components/InputStream.md)
- [`View`](./components/View.md)
- [`Rescaler`](./components/Rescaler.md)
- [`Tiles`](./components/Tiles.md)
- [`Text`](./components/Text.md)
- [`Shader`](./components/Shader.md)
- [`Image`](./components/Image.md)
- [`InputStream`](./components/WebView.md)

You can't use DOM components like `<div/>` when composing streams. React code can only use LiveCompositor specific components.

## Hooks

React hooks that can be used in React code that controls stream composition.

- [`useInputStreams`](./hooks.md#useinputstreams)
- [`useInputAudio`](./hooks.md#useinputaudio)

You can also use regular React hooks like `useState`, `useEffect` and others.

## Renderers

Functionality that can be used when composing streams, but it has to be registered first e.g. `Shader` needs to be registered first before you use `<Shader>` component.

- [`Shader`](./renderers/shader.md)
- [`Image`](./renderers/image.md)
- [`WebRenderer`](./renderers/web.md)

## Inputs

To deliver video/audio to the compositor you need to register some inputs. Registered stream can be used in composition using `<InputStream inputId="example_input" />` component. 

- [`RTP`](./inputs/rtp.md)
- [`MP4`](./inputs/mp4.md)

You can register an input with [`LiveCompositor.registerInput`](./instance#register-input)

## Outputs

Defines protocol, format and destination of the composed video and mixed audio streams.

- [`RTP`](./outputs/rtp.md)
- [`MP4`](./outputs/mp4.md)

You can register an output with [`LiveCompositor.registerOutput`](./instance#register-output)
