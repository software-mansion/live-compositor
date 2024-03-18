---
description: API routes to configure the compositor.
---

# Routes

API is served by default on the port 8081. Different port can be configured using [`LIVE_COMPOSITOR_API_PORT`](../deployment/configuration#live_compositor_api_port) environment variable.

## Endpoint `POST /--/api`

Main endpoint for configuring the compositor server.

### Start

```typescript
type Start = {
  type: "start";
}
```

Starts the processing pipeline. If outputs are registered and defined in the scene then the compositor will start to send the RTP stream.

***

### Update output

```typescript
type UpdateOutput = {
  type: "update_output";
  output_id: string;
  video?: Component;
  audio?: {
    inputs: AudioInput[];
    mixing_strategy: "sum_clip" | "sum_scale"
  };
  schedule_time_ms?: number;
}

type AudioInput = {
  input_id: InputId;
  volume?: number;
}
```

- `output_id` - Id of an already registered output stream. See [`RegisterOutputStream`](./routes#register-output-stream).
- `video` - Root of a component tree/scene that should be rendered for the output. [Learn more](../concept/component)
- `audio` - Parameters for mixing input audio streams.
- `audio.inputs[].input_id` - Input id.
- `audio.inputs[].volume` - (**default=`1.0`**) Float in `[0, 1]` range representing volume.
- `audio.mixing_strategy` - (**default=`sum_clip`**) Strategy for mixing audio:
- - `"sum_clip"` - Clip summed input waves if it's outside the PCM range.
- - `"sum_scale"` - Scales down summed input waves if it's outside the PCM range.
- `schedule_time_ms` - Time in milliseconds when this request should be applied. Value `0` represents time of [the start request](#start).

***

### Register input stream

```typescript
type RegisterInputStream = {
  type: "register";
  entity_type: "rtp_input_stream" | "mp4";
  ... // input specific options
}
```

Register external source that can be used as a compositor input. See inputs documentation to learn more.

- [RTP](./inputs/rtp.md)
- [MP4](./inputs/mp4.md)

***

### Register output stream

```typescript
type RegisterOutputStream = {
  type: "register";
  entity_type: "output_stream";
  ...
}
```

Register external destination that can be used as a compositor output. See outputs documentation to learn more.

- [RTP](./outputs/rtp.md)

***

### Register renderer

```typescript
type RegisterRenderer = {
  type: "register";
  entity_type: "shader" | "web_renderer" | "image";
  ... // renderer specific options
}
```

See renderers documentation to learn more.

- [Image](./renderers/image)
- [Shader](./renderers/shader)
- [WebRenderer](./renderers/web)

***

### Unregister request

```typescript
type Unregister =
  | {
    type: "unregister";
    entity_type: "input_stream";
    input_id: string;
    schedule_time_ms: number;
  }
  | {
    type: "unregister";
    entity_type: "output_stream";
    output_id: string;
    schedule_time_ms: number;
  }
  | { type: "unregister"; entity_type: "shader"; shader_id: string }
  | { type: "unregister"; entity_type: "image"; image_id: string }
  | { type: "unregister"; entity_type: "web_renderer"; instance_id: string }
```

Removes entities previously registered with [register input](#register-input-stream), [register output](#register-output-stream) or [register renderer](#register-renderer) requests.

- `schedule_time_ms` - Time in milliseconds when this request should be applied. Value `0` represents time of [the start request](#start).

## Endpoint `GET /status`

Status/health check endpoint. Returns `200 OK`.
