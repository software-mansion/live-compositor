---
description: API routes to configure the compositor.
---

# Routes

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
  port: u16 | string;
}
```

Register a new RTP input stream.

- `input_id` - An identifier for the input stream. It can be used in the [`InputStream`](./components/InputStream) component to render the stream content.
- `port` - UDP port or port range on which the compositor should listen for the stream. An integer value between 1 and 65535 that represents a specific port
or string in the `START:END` format for a port range.

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
  encoder_settings: {
    preset: EncoderPreset;
  };
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
- `encoder_settings.preset` - Preset for an encoder. See `FFmpeg` [docs](https://trac.ffmpeg.org/wiki/Encode/H.264#Preset) to learn more.

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
