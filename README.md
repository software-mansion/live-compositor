# LiveCompositor

![LiveCompositor logo](./assets/lc_logo_large.svg)

LiveCompositor is an open-source media server for real-time, low-latency, programmable video and audio mixing/composing.

LiveCompositor targets real-time use cases, like video conferencing, live-streaming, or broadcasting (e.g. with [WebRTC](https://en.wikipedia.org/wiki/WebRTC) / [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming) / [RTMP](https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol)), however [offline processing](https://compositor.live/docs/deployment/configuration#live_compositor_offline_processing_enable) is also available.

<!-- TODO change offline processing url to offline processing guide -->

We don't have plans to introduce any major breaking changes in the API in the forseeable future.

## Features

LiveCompositor receives inputs and sends output streams via [RTP](https://en.wikipedia.org/wiki/Real-time_Transport_Protocol).
HTTP API is used to define how inputs should be transformed and combined to produce desired outputs.

High-level, web-style API, similar to HTML, can be used to layout and mix:

- live RTP input streams
- MP4s
- images / GIFs
- text
- color backgrounds
- websites (experimental, rendered with Chromium embedded in LiveCompositor, not recommended for high-performance/production usage)

Dynamic output layout transitions are directly supported.
For any custom effects, users can [register and use their own WGSL shaders](https://compositor.live/docs/concept/shaders).

Input audio can be mixed directly in LiveCompositor, providing synchronization for video/audio outputs.

## Demos

https://github.com/membraneframework/live_compositor/assets/104033489/e6f5ba7c-ab05-4935-a42a-bc28c42fc895

TypeScript demos presenting what you can do with LiveCompositor are available in the `demos` directory.

## Examples

Examples source code showcasing single API components usage are available in the `examples` directory.

Running examples requires:

- [Rust](https://www.rust-lang.org/tools/install)
- [FFmpeg 6.0](https://ffmpeg.org/download.html)

For better performance, build examples with the [release compilation profile](https://doc.rust-lang.org/book/ch14-01-release-profiles.html):

```console
cargo run --release --example <example_name>
```

## Guides

Step-by-step introduction guides are available on the [LiveCompositor website](https://compositor.live/docs/guides).

## Supported platforms

Linux and macOS.

## Current development

Currently, we are working on:

- hardware decoding/encoding implementation for Vulcan (for better performance)
- guides/documentation/deployment improvements
- supporting more features, like corners-rounding

## Copyright

Copyright 2023, [Software Mansion](https://swmansion.com/?utm_source=git&utm_medium=readme&utm_campaign=live_compositor)

[![Software Mansion](https://logo.swmansion.com/logo?color=white&variant=desktop&width=200&tag=membrane-github)](https://swmansion.com/?utm_source=git&utm_medium=readme&utm_campaign=live_compositor)
