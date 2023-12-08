---
description: API routes to configure the compositor.
---

# Routes

### Init

```typescript
Init = {
  type: "init",
  web_renderer: WebRendererOptions,
  framerate: number,
  stream_fallback_timeout_ms?: number // default: 1000
}
```

Init request triggers the initial setup of a compositor. It defines the base settings of a compositor that need to be evaluated before any other work happens.

- `web_renderer` - Web renderer specific options. Read more [here](https://github.com/membraneframework/video_compositor/wiki/Web-renderer#global-options).
- `framerate` - Target framerate of the output streams.
- `stream_fallback_timeout_ms` (default: 1000) - Timeout that defines when the compositor should switch to fallback on the input stream that stopped sending frames. See [fallback](https://github.com/membraneframework/video_compositor/wiki/Main-concepts#fallback) to learn more.

***

### Start

```typescript
Start = {
  type: "start"
}
```

Starts the processing pipeline. If outputs are registered and defined in the scene then the compositor will start to send the RTP stream.

***

### Update scene

```typescript
UpdateScene = {
  type: "update_scene",
  nodes: Array<Node>,
  outputs: Array<Output>,
}

Node = {
  type: "shader" | "web_renderer" | "text_renderer" | "built-in",
  node_id: NodeId,
  input_pads: Array<NodeId>,
  fallback_id?: NodeId,
  ...
}

Output = {
  output_id: string,
  input_pad: NodeId,
}

NodeId = string
```

Update [scene](https://github.com/membraneframework/video_compositor/wiki/Main-concepts#scene).

- `nodes` - List of nodes in the pipeline. Each node defines how inputs are converted into an output.
  - `nodes[].node_id` - Id of a node.
  - `nodes[].input_pads` - List of node ids that identify nodes needed for a current node to render. The actual meaning of that list depends on specific node implementation.
  - `nodes[].fallback_id` - Id of a node that will be used instead of the current one if [fallback](https://github.com/membraneframework/video_compositor/wiki/Main-concepts#fallback) is triggered.
  - `nodes[].*` - other params depend on the node type. See [Api - nodes](https://github.com/membraneframework/video_compositor/wiki/API-%E2%80%90-nodes) for more.
- `outputs` - List of outputs. Identifies which nodes should be used to produce RTP output streams.
  - `outputs[].output_id` - Id of an already registered output stream. See `RegisterOutputStream`.
  - `outputs[].input_pad` - Id of a node that will be used to produce frames for output `output_id`.

***

### Register input stream

```typescript
RegisterInputStream = {
  type: "register",
  entity_type: "input_stream",
  input_id: string,
  port: number
}
```

Register a new [input stream](https://github.com/membraneframework/video_compositor/wiki/Main-concepts#inputoutput-streams).

- `input_id` - Identifier that can be used in `UpdateScene` request to connect that stream to transformations or outputs.
- `port` - UDP port that the compositor should listen for stream.

***

### Register output stream

```typescript
RegisterOutputStream = {
  type: "register",
  entity_type: "output_stream",
  output_id: string,
  port: number,
  ip: string,
  resolution: {
    width: number,
    height: number,
  },
  encoder_settings: {
    preset: EncoderPreset
  }
}

EncoderPreset =
  | "ultrafast"
  | "superfast"
  | "veryfast"
  | "faster"
  | "fast"
  | "medium"
  | "slow"
  | "slower"
  | "veryslow"
  | "placebo"
```

Register a new [output stream](https://github.com/membraneframework/video_compositor/wiki/Main-concepts#inputoutput-streams).

- `output_id` - Identifier that can be used in `UpdateScene` request to assign a node that will be used to produce frames for a stream.
- `port` / `ip` - UDP port and IP where compositor should send the stream. 
- `resolution` - Output resolution in pixels.
- `encoder_settings.preset` - Preset for an encoder. See `FFmpeg` [docs](https://trac.ffmpeg.org/wiki/Encode/H.264#Preset) to learn more.

### Register renderer

```typescript
RegisterRenderer = {
  type: "register",
  entity_type: "shader" | "web_renderer" | "image",
  ... // renderer specific options
}
```

See [renderer documentation](https://github.com/membraneframework/video_compositor/wiki/Api-%E2%80%90-renderers) to learn more.

### Unregister request

```typescript
Unregister =
  | { entity_type: "input_stream", input_id: string }
  | { entity_type: "output_stream", output_id: string }
  | { entity_type: "shader", shader_id: string }
  | { entity_type: "image", image_id: string }
  | { entity_type: "web_renderer", instance_id: string };
```
