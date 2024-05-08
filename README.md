# LiveCompositor

Application for real-time video processing/transforming/composing, providing simple, language-agnostic API for live video rendering.

LiveCompositor targets real-time use cases, like video conferencing, live-streaming, or broadcasting (e.g. with [WebRTC](https://en.wikipedia.org/wiki/WebRTC) / [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming) / [RTMP](https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol)).

## Features

LiveCompositor receives inputs and sends outputs streams via [RTP](https://en.wikipedia.org/wiki/Real-time_Transport_Protocol).
HTTP API is used to define how inputs should be transformed and combined to produce desired outputs.

For the initial release, we want LiveCompositor to support those four types of transformations, that you can combine:

- Common transformations - frequently used, already implemented transformations, like layouts, grids, cropping, corners rounding, blending, fading, etc.
- Custom shader transformations - registering and using custom shaders, allowing to adapt LiveCompositor for specific business needs.
- Web Rendering - embedding videos in custom websites.
- Text Rendering

## Demos

TypeScript demos presenting what you can do with LiveCompositor are available in the `demos` directory.

## Examples

Examples source code is under the `examples` directory.

Running examples requires:

- [Rust](https://www.rust-lang.org/tools/install)
- [FFmpeg 6.0](https://ffmpeg.org/download.html)

For better performance, build examples with the [release compilation profile](https://doc.rust-lang.org/book/ch14-01-release-profiles.html):

```console
cargo run --release --example <example_name>
```

## Supported platforms

Linux and macOS.

## Copyright

Copyright 2023, [Software Mansion](https://swmansion.com/?utm_source=git&utm_medium=readme&utm_campaign=live_compositor)

[![Software Mansion](https://logo.swmansion.com/logo?color=white&variant=desktop&width=200&tag=membrane-github)](https://swmansion.com/?utm_source=git&utm_medium=readme&utm_campaign=live_compositor)
