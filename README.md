# VideoCompositor

Application for real-time video processing/transforming/composing.
Provides simple, language-agnostic, API for live video rendering.

It's designed for real-time use cases, like video conferencing, live-streaming, or broadcasting (e.g. with [WebRTC](https://en.wikipedia.org/wiki/WebRTC) / [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming) / [RTMP](https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol)).

## Features

VideoCompositor receives inputs and sends outputs streams via [RTP](https://en.wikipedia.org/wiki/Real-time_Transport_Protocol). Processing specification is represented as a `Scene` graph sent in JSON format via HTTP request. 

The `Scene` contains `input` nodes (inputs streams frames), `transform` nodes (applying effects to frames), and `output` nodes (sending frames in output streams).
Currently, we want VideoCompositor to have the following `transform` types possible in the `Scene`:

- Common video transformations - frequently used, already implemented transformations, like layouts, grids, cropping, corners rounding, blending, fading, etc.
- Custom shader transformations - registering and using custom shaders, allowing to adapt VideoCompositor for specific business needs
- Web Rendering - embedding videos in custom websites
- Text Rendering

## Copyright

Copyright 2023, [Software Mansion](https://swmansion.com/?utm_source=git&utm_medium=readme&utm_campaign=video_compositor)

[![Software Mansion](https://logo.swmansion.com/logo?color=white&variant=desktop&width=200&tag=membrane-github)](https://swmansion.com/?utm_source=git&utm_medium=readme&utm_campaign=video_compositor)