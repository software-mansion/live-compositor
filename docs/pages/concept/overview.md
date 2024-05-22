# Concepts

## Inputs/outputs streams

LiveCompositor receives inputs and sends output streams via [RTP](https://en.wikipedia.org/wiki/Real-time_Transport_Protocol).
Additionally, you can also use MP4 files directly as an input. To deliver/receive any other formats you can use tools like
FFmpeg, GStreamer, or Membrane Framework to convert between RTP end the desired format.

## Video composition

For each output stream, you need to define a scene. A scene is a tree-like structure of components that defines what should be
rendered. This component tree API should be easy to pick up for anyone familiar with Web development.

You can construct a scene from, among other things, the following components:
- [`InputStream`](../api/components/InputStream.md) - RTP stream or MP4 file.
- [`View`](../api/components/View.md) - The most basic/core component, an analog of `<div>` in HTML or `<View>` in React Native.
- [`Rescaler`](../api/components/Rescaler.md) - Resizes its child component.
- [`Image`](../api/components/Image.md) - Supports PNG, JPEG, SVG, and GIF (including animated ones).
- [`Text`](../api/components/Text.md)
- [`WebView`](../api/components/WebView.md) - Renders a website using a browser.

Learn more about components and the scene [here](./component.md).

## Audio mixer

LiveCompositor supports audio streams and provides a simple mixer to combine them. Even if you only have one audio stream and do not need
to modify it in any way, then it is still good to pass that stream to the compositor to avoid synchronization issues between audio and video.

