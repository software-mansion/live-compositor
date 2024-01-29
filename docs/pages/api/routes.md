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
  outputs: OutputScene[];
}

type OutputScene = {
    output_id: string;
    root: Component;
}
```

- `outputs` - List of outputs. Identifies what should be rendered for each RTP output streams.
  - `outputs[].output_id` - Id of an already registered output stream. See [`RegisterOutputStream`](./routes#register-output-stream).
  - `outputs[].root` - Root of a component tree that should be rendered for the output. [Learn more](../concept/component)

***

### Register input stream

```typescript
type RegisterInputStream = {
  type: "register";
  entity_type: "input_stream";
  input_id: string;
  port: Port;
  video?: Video;
  audio?: Audio;
}
```

Parameters of registered RTP input stream. Before using input in video composition or output mixing, input has to be firstly registered using `register_input` request.

At least one of `video` and `audio` has to be defined.

- `input_id` - An identifier for the input stream.
- `port` - UDP port or port range on which the compositor should listen for the stream.
- `video` - Parameters of a video source included in the RTP stream.
- `audio` - Parameters of an audio source included in the RTP stream.

#### Port

```typescript
type Port = string | u16
```

#### Video

```typescript
type Video = {
  codec?: "h264";
  rtp_payload_type?: u8;
}
```

- `codec` - (**default=`"h264"`**) Video codec.
  - `"h264"` - H264 video.
- `rtp_payload_type` - (**default=`96`**) Value of payload type field in received RTP packets.
  Packets with different payload type won't be treated as video and included in composing. Values should be in [0, 64] or [96, 255]. Values in range [65, 95] can't be used. For more information, see [RFC](https://datatracker.ietf.org/doc/html/rfc5761#section-4) Packets with different payload type won't be treated as video and included in composing.

#### Audio

```typescript
type Audio = {
  codec?: "opus";
  sample_rate: u32;
  channels: "mono" | "stereo";
  rtp_payload_type?: u8;
  forward_error_correction?: bool;
}
```

- `codec` - (**default=`"opus"`**) Audio codec.
  - `"opus"` - Opus audio.
- `sample_rate` - Sample rate. If the specified sample rate doesn't match real sample rate, audio won't be mixed properly.
- `channels` - Audio channels.
  - `"mono"` - Mono audio (single channel).
  - `"stereo"` - Stereo audio (two channels).
- `rtp_payload_type` - (**default=`97`**) Value of payload type field in received RTP packets.
  Packets with different payload type won't be treated as audio and included in mixing. Values should be in range [0, 64] or [96, 255]. Values in range [65, 95] can't be used. For more information, check out [RFC](https://datatracker.ietf.org/doc/html/rfc5761#section-4).
- `forward_error_correction` - (**default=`"false"`**) Specifies whether the stream uses forward error correction. It's specific for Opus codec. For more information, check out [RFC](https://datatracker.ietf.org/doc/html/rfc6716#section-2.1.7).

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
