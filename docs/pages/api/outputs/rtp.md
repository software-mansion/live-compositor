# RTP

An output type that allows streaming video and audio from the compositor over RTP.

```typescript
type RegisterOutputStream = {
  output_id: string;
  transport_protocol?: "udp" | "tcp_server";
  port: u16;
  ip?: string;
  video?: Video;
  audio?: Audio;
}
```

Register a new RTP output stream.

- `output_id` - An identifier for the output stream. It can be used in the `UpdateOutput` request to define what to render for the output stream.
- `transport_protocol` - (**default=`"udp"`**) Transport layer protocol that will be used to send RTP packets.
  - `udp` - UDP protocol.
  - `tcp_server` - TCP protocol where LiveCompositor is the server side of the connection.
- `port` - Depends on the value of the `transport_protocol` field:
  - `udp` - An UDP port number that RTP packets will be sent to.
  - `tcp_server` - A local TCP port number or a port range that LiveCompositor will listen for incoming connections.
- `ip` - Only valid if `transport_protocol="udp"`. IP address where RTP packets should be sent to.

```typescript
type Video = {
  resolution: { width: number; height: number };
  encoder_preset?: VideoEncoderPreset;
  ffmpeg_options?: Map<String, String>;
  send_eos_when?: EosCondition;
  initial: Component;
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

- `resolution` - Output resolution in pixels.
- `encoder_preset` - (**default=`"fast"`**) Preset for an encoder. See `FFmpeg` [docs](https://trac.ffmpeg.org/wiki/Encode/H.264#Preset) to learn more.
- `ffmepg_options` - Raw FFmpeg encoder options. See [docs](https://ffmpeg.org/ffmpeg-codecs.html) for more.
- `send_eos_when` - Defines when output stream should end if some of the input streams are finished. If output includes both audio and video streams, then EOS needs to be sent on both.
- `initial` - Root of a component tree/scene that should be rendered for the output. Use [`update_output` request](../routes.md#update-output) to update this value after registration. [Learn more](../../concept/component.md).



```typescript
type Audio = {
  channels: "stereo" | "mono";
  forward_error_correction?: boolean;
  encoder_preset?: AudioEncoderPreset;
  send_eos_when?: EosCondition;
  initial: {
    inputs: AudioInput[];
  };
  mixing_strategy?: "sum_clip" | "sum_scale" 
}

type AudioInput = {
  input_id: string;
  volume?: number;
}

type AudioEncoderPreset =
  | "quality"
  | "voip"
  | "lowest_latency"

```
- `channels` - Channel configuration for output audio.
- `forward_error_correction` - (**default=`false`**) Specifies whether the stream use forward error correction. It's specific for Opus codec. For more information, check out [RFC](https://datatracker.ietf.org/doc/html/rfc6716#section-2.1.7).
- `encoder_preset` - (**default=`"voip"`**) Preset for an encoder.
  - `quality` - Best for broadcast/high-fidelity application where the decoded audio should be as close as possible to the input.
  - `voip` - Best for most VoIP/videoconference applications where listening quality and intelligibility matter most.
  - `lowest_latency` - Only use when lowest-achievable latency is what matters most.
- `send_eos_when` - Defines when output stream should end if some of the input streams are finished. If output includes both audio and video streams, then EOS needs to be sent on both.
- `initial` - Initial configuration for audio mixer for this output. Use [`update_output` request](../routes.md#update-output) to update this value after registration.
- `initial.inputs[].input_id` - Input ID.
- `initial.inputs[].volume` - (**default=`1.0`**) Float in `[0, 1]` range representing volume.
- `mixing_strategy` - (**default=`sum_clip`**) Specifies how input samples should be mixed:
  - `sum_clip` - Firstly, input samples are summed. If the result sample is outside the i16 PCM range, it gets clipped.
  - `sum_scale` - Firstly, input samples are summed. If the result wave is outside the i16 PCM range, nearby samples are scaled down by factor, such that the summed wave is in the i16 PCM range.

```typescript
type EosCondition = {
  any_input?: bool;
  all_inputs?: bool;
  any_of?: InputId[];
  all_of?: InputId[];
}
```

This type defines when end of an input stream should trigger end of the output stream. Only one of those fields can be set at the time.

Unless specified otherwise the input stream is considered finished/ended when:
- TCP connection was dropped/closed.
- RTCP Goodbye packet (`BYE`) was received.
- Mp4 track has ended.
- Input was unregistered already (or never registered).

Options:
- `any_of` - Terminate output stream if any of the input streams from the list are finished.
- `all_of` - Terminate output stream if all the input streams from the list are finished.
- `any_input` - Terminate output stream if any of the input streams ends. This includes streams added after the output was registered. In particular, output stream will **not be** terminated if no inputs were ever connected.
- `all_inputs` - Terminate output stream if all the input streams finish. In particular, output stream will **be** terminated if no inputs were ever connected.

