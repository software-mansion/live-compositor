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
  video?: Video;
  audio?: Audio;
}

type Video = {
  resolution: { width: number, height: number },
  encoder_preset?: VideoEncoderPreset,
  initial: Component,
}

type Audio = {
  channels: "stereo" | "mono";
  forward_error_correction?: boolean;
  encoder_preset?: AudioEncoderPreset;
  initial: {
    inputs: AudioInput[];
  };
}

type AudioInput = {
  input_id: string;
  volume?: number;
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

type AudioEncoderPreset =
  | "quality"
  | "voip"
  | "lowest_latency"
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
- `video.encoder_preset` - (**default=`"fast"`**) Preset for an encoder. See `FFmpeg` [docs](https://trac.ffmpeg.org/wiki/Encode/H.264#Preset) to learn more.
- `video.initial` - Root of a component tree/scene that should be rendered for the output. Use [`update_output` request](#update-output) to update this value after registration. [Learn more](../concept/component).

- `audio.channels` - Channel configuration for output audio.
- `audio.forward_error_correction` - (**default=`false`**) Specifies whether the stream use forward error correction. It's specific for Opus codec. For more information, check out [RFC](https://datatracker.ietf.org/doc/html/rfc6716#section-2.1.7).
- `audio.encoder_preset` - (**default=`"voip"`**) Preset for an encoder.
  - `quality` - Best for broadcast/high-fidelity application where the decoded audio should be as close as possible to the input.
  - `voip` -  Best for most VoIP/videoconference applications where listening quality and intelligibility matter most.
  - `lowest_latency` - Only use when lowest-achievable latency is what matters most.
- `audio.initial` - Initial configuration for audio mixer for this output. Use [`update_output` request](#update-output) to update this value after registration.
- `audio.initial.inputs[].input_id` - Input id.
- `audio.initial.inputs[].volume` - (**default=`1.0`**) Float in `[0, 1]` range representing volume.

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
