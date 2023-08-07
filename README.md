# VideoCompositor

Application for real-time video processing/transforming/composing, providing simple, language-agnostic API for live video rendering.

VideoCompositor targets real-time use cases, like video conferencing, live-streaming, or broadcasting (e.g. with [WebRTC](https://en.wikipedia.org/wiki/WebRTC) / [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming) / [RTMP](https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol)).

## Features

VideoCompositor receives inputs and sends outputs streams via [RTP](https://en.wikipedia.org/wiki/Real-time_Transport_Protocol). Processing specification is represented as a `Scene` graph.
Actions, such as registering inputs, updating `Scene`, and the like, are sent via HTTP request.

The `Scene` contains `input` nodes (inputs streams frames), `transform` nodes (applying effects to frames), and `output` nodes (sending frames in output streams).
Currently, we want VideoCompositor to have the following `transform` types possible in the `Scene`:

- Common video transformations - frequently used, already implemented transformations, like layouts, grids, cropping, corners rounding, blending, fading, etc.
- Custom shader transformations - registering and using custom shaders, allowing to adapt VideoCompositor for specific business needs
- Web Rendering - embedding videos in custom websites
- Text Rendering

## Examples

Examples source code is located in the `examples` directory.

### Installation

Examples use [`FFmpeg`](https://www.ffmpeg.org/) for sending and receiving RTP input/output streams.

#### Rust installation

To install rust run:

```console
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### FFmpeg installation

##### Ubuntu

```console
sudo apt-get install libavcodec-dev libavformat-dev libavutil-dev
```

##### Arch/Manjaro

```console
pacman -S ffmpeg
```

##### MacOS

```console
brew install ffmpeg
```

### Running

For better performance, build examples with the [release compilation profile](https://doc.rust-lang.org/book/ch14-01-release-profiles.html):

```console
cargo run --release --example <example_name>
```

## Supported platforms

Currently, we support Linux and MacOS.

## Copyright

Copyright 2023, [Software Mansion](https://swmansion.com/?utm_source=git&utm_medium=readme&utm_campaign=video_compositor)

[![Software Mansion](https://logo.swmansion.com/logo?color=white&variant=desktop&width=200&tag=membrane-github)](https://swmansion.com/?utm_source=git&utm_medium=readme&utm_campaign=video_compositor)
