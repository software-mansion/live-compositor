# How to use it?

Live Compositor can be used or deployed in a few ways.

## TypeScript/React

TypeScript SDK currently can only be run in Node.js environment, but browser and React Native support will be added soon. Run
```
npm create live-compositor
```
to generate a new starter project.

There are 2 NPM packages that you need to be aware of:
- `live-compositor` package provides React components and hooks to define how streams should be composed.
- `@live-compositor/node` package provides interface to interact with the Live Compositor server from Node.js environment.

See [TypeScript SDK documentation for more](../typescript/api.md) to learn more.

## Standalone

You can use LiveCompositor as a standalone multimedia server. The server can be started by:
- Building [`github.com/software-mansion/live-compositor`](https://github.com/software-mansion/live-compositor) from source.
- Using binaries from [GitHub releases](https://github.com/software-mansion/live-compositor/releases).
- Using Docker
  - (recommended) Dockerfile with compositor without web rendering support [https://github.com/software-mansion/live-compositor/blob/master/build_tools/docker/slim.Dockerfile](https://github.com/software-mansion/live-compositor/blob/master/build_tools/docker/slim.Dockerfile)
  - Dockerfile with compositor with web rendering support [https://github.com/software-mansion/live-compositor/blob/master/build_tools/docker/full.Dockerfile](https://github.com/software-mansion/live-compositor/blob/master/build_tools/docker/full.Dockerfile)

## Membrane Framework plugin

Membrane Framework has its own way of handling multimedia, so to fit into that ecosystem some features do not translate one-to-one between standalone compositor and the plugin.

Notable differences:
- Inputs/outputs in LiveCompositor can include both audio and video at the same time, but with the Membrane plugin you need to create separate inputs/outputs for each media type.
- No support for MP4 files as input. It is more idiomatic to use Membrane plugins to read MP4 files instead.
- To connect inputs/outputs to LiveCompositor you need to first register them before sending/receiving the stream, but with the Membrane plugin connecting pads covers both those steps.

Parts of this documentation were written with a standalone scenario in mind, so make sure to always consult [the plugin documentation](https://hexdocs.pm/membrane_live_compositor_plugin/0.9.0/Membrane.LiveCompositor.html) first. For example, to see how to send a scene update check out documentation on `HexDocs`, but if you want to know what options the `View` component supports, then consult the documentation [here](../api/components/View.md).
