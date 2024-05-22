# Getting started

## What is Live Compositor?

LiveCompositor is an open-source media server for real-time, low-latency, programmable video and audio mixing. 

LiveCompositor targets real-time use cases, with a significant focus on situations where latency is critical. It is a great fit
for any video conferencing, live-streaming, or broadcasting solutions where you need to operate in any way on an uncompressed video.
However, you can also use it for non-real-time use cases, for example, apply some effect on a video from an MP4 file and write the output 
to file as MP4.

## Where to start?

To get started check out our [`Guides`](./category/guides) section that will walk you through common scenarios.
- [`Simple scene`](./guides/simple-scene.md) describes how to achieve a few of the most basic layouts when composing video.
- [`Deliver input streams`](./guides/deliver-input.md) explains and shows examples of streaming multimedia to the LiveCompositor and use them for mixing/composition.
- [`Receive output streams`](./guides/receive-output.md) explains and shows examples of receiving streams with results of mixing/composition from the LiveCompositor

The main concept and basic abstractions that the LiveCompositor operates on are described in the [`Concepts`](/docs/category/concepts) section.

## How to use it?

Live Compositor can be used standalone or as a part of a Membrane Framework multimedia pipeline.

### Standalone

You can use LiveCompositor as a standalone multimedia server. The server can be started by:
- Building [`github.com/membraneframework/live_compositor`](https://github.com/membraneframework/live_compositor) from source.
- Using binaries from [GitHub releases](https://github.com/membraneframework/live_compositor/releases).
- Using Docker
  - (recommended) Dockerfile with compositor without Whttps://github.com/membraneframework/live_compositor/blob/master/build_tools/docker/slim.Dockerfile

### Membrane Framework plugin

Membrane Framework has its own way of handling multimedia, so to fit into that ecosystem some features do not translate one-to-one between standalone compositor and the plugin.

Notable differences:
- Inputs/outputs in LiveCompositor can include both audio and video at the same time, but with the Membrane plugin you need to create separate inputs/outputs for each media type.
- No support for MP4 files as input. It is more idiomatic to use Membrane plugins to read MP4 files instead.
- To connect inputs/outputs to LiveCompositor you need to first register them before sending/receiving the stream, but with the Membrane plugin connecting pads covers both those steps.

Parts of this documentation were written with a standalone scenario in mind, so make sure to always consult [the plugin documentation](https://github.com/membraneframework/membrane_live_compositor_plugin) first. For example, to see how to send a scene update check out documentation on `HexDocs`, but if you want to know what options the `View` component supports, then consult the documentation [here](./api/components/View.md).
