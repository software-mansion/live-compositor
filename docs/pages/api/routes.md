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
  audio?: AudioMixParams
}

type AudioMixParams = {
  inputs: [InputAudioParams]
}

type InputAudioParams = {
  input_id: InputId
}
```

- `output_id` - Id of an already registered output stream. See [`RegisterOutputStream`](./routes#register-output-stream).
- `video` - Root of a component tree/scene that should be rendered for the output. [Learn more](../concept/component)
- `audio` - Parameters for mixing input audio streams.

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
  transport_protocol?: "udp" | "tcp_server";
  port: u16;
  ip?: string;
  video: Video
}

type Video = {
  resolution: {
    width: number,
    height: number
  },
  initial: Component
  encoder_preset?: VideoEncoderPreset,
}

type VideoEncoderPreset =
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

- `output_id` - An identifier for the output stream. It can be used in the `UpdateOutput` request to define what to render for the output stream.
- `transport_protocol` -  (**default=`"udp"`**) Transport layer protocol that will be used to send RTP packets.
  - `udp` - UDP protocol.
  - `tcp_server` - TCP protocol where LiveCompositor is the server side of the connection.
- `port` - Depends on the value of the `transport_protocol` field:
  - `udp` - An UDP port number that RTP packets will be sent to.
  - `tcp_server` - A local TCP port number or a port range that LiveCompositor will listen for incoming connections.
- `ip` - Only valid if `transport_protocol="udp"`. IP address where RTP packets should be sent to.
- `video.resolution` - Output resolution in pixels.
- `video.initial` - Root of a component tree/scene that should be rendered for the output. Use [`update_output` request](#update-output) to update this value after registration. [Learn more](../concept/component).
- `video.encoder_preset` - (**default=`"fast"`**) Preset for an encoder. See `FFmpeg` [docs](https://trac.ffmpeg.org/wiki/Encode/H.264#Preset) to learn more.

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
