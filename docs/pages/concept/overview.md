# Concepts

## Inputs/outputs streams

LiveCompositor receives inputs and sends output streams via [RTP](https://en.wikipedia.org/wiki/Real-time_Transport_Protocol).
Additionally, you can also use MP4 files directly as an input. To deliver/receive any other formats you can use tools like
FFmpeg, GStreamer, or Membrane Framework to convert between RTP and the desired format.

## Video composition

For each output stream, you need to define a scene. A scene is a tree-like structure of components that defines what should be
rendered. This component tree API should be easy to pick up for anyone familiar with Web development.

You can construct a scene from, among other things, the following components:
- `InputStream` - RTP stream or MP4 file. ([`TypeScript`](../typescript/components/InputStream.md), [`HTTP`](../api/components/InputStream.md))
- `View` - The most basic/core component, an analog of `<div>` in HTML or `<View>` in React Native. ([`TypeScript`](../typescript/components/View.md), [`HTTP`](../api/components/View.md))
- `Rescaler` - Resizes its child component. ([`TypeScript`](../typescript/components/Rescaler.md), [`HTTP`](../api/components/Rescaler.md))
- `Image` - Supports PNG, JPEG, SVG, and GIF (including animated ones). ([`TypeScript`](../typescript/components/Image.md), [`HTTP`](../api/components/Image.md))
- `Text` ([`TypeScript`](../typescript/components/Text.md), [`HTTP`](../api/components/Text.md))
- `WebView` - Renders a website using a browser. ([`TypeScript`](../typescript/components/WebView.md), [`HTTP`](../api/components/WebView.md))

Learn more about components and the scene [here](./component.md).

## Audio mixer

LiveCompositor supports audio streams and provides a simple mixer to combine them. Even if you only have one audio stream and do not need
to modify it in any way, then it is still good to pass that stream to the compositor to avoid synchronization issues between audio and video.

