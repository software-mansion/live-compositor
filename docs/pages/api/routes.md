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

### Update scene

```typescript
type UpdateScene = {
  type: "update_scene";
  output_id: string;
  scene: Component;
}
```

- `output_id` - Id of an already registered output stream. See [`RegisterOutputStream`](./routes#register-output-stream).
- `scene` - Root of a component tree/scene that should be rendered for the output. [Learn more](../concept/component)

***

### Register input stream

```typescript
type RegisterInputStream = {
  type: "register";
  entity_type: "rtp_input_stream" | "mp4";
  ... // input specific options
}
```
See inputs documentation to learn more.

- [RTP](./inputs/rtp)
- [MP4](./inputs/mp4)

***

### Register output stream

```typescript
type RegisterOutputStream = {
  type: "register";
  entity_type: "output_stream";
  output_id: string;
  port: u16;
  ip: string;
  resolution: {
    width: number;
    height: number;
  };
  encoder_preset?: EncoderPreset;
  initial_scene: Component
}

type EncoderPreset =
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

Register a new RTP output stream.

- `output_id` - An identifier for the output stream. It can be used in the `UpdateScene` request to define what to render for the output stream.
- `port` / `ip` - UDP port and IP where compositor should send the stream.
- `resolution` - Output resolution in pixels.
- `encoder_preset` - (**default=`"fast"`**) Preset for an encoder. See `FFmpeg` [docs](https://trac.ffmpeg.org/wiki/Encode/H.264#Preset) to learn more.
- `initial_scene` - Root of a component tree/scene that should be rendered for the output. Use [`update_scene` request](#update-scene) to update this value after registration. [Learn more](../concept/component).

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
  | { type: "unregister", entity_type: "input_stream", input_id: string }
  | { type: "unregister", entity_type: "output_stream", output_id: string }
  | { type: "unregister", entity_type: "shader", shader_id: string }
  | { type: "unregister", entity_type: "image", image_id: string }
  | { type: "unregister", entity_type: "web_renderer", instance_id: string }
```

## Endpoint `GET /status`

Status/health check endpoint. Returns `200 OK`.
