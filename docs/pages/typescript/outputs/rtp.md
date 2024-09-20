---
title: RTP
---
import Docs from "@site/pages/api/generated/output-RtpOutputStream.md"

# RTP

An output type that allows streaming video and audio from the compositor over RTP.

### `Outputs.RegisterRtpOutput` 

```typescript
import { Outputs } from "live-compositor"

type RtpOutputStream = {
  port: string | number;
  ip?: string;
  transportProtocol?: "udp" | "tcp_server";
  video?: Outputs.RtpVideoOptions;
  audio?: Outputs.RtpAudioOptions;
}
```

#### Properties
- `port` - Depends on the value of the `transportProtocol` field:
  - `udp` - An UDP port number that RTP packets will be sent to.
  - `tcp_server` - A local TCP port number or a port range that LiveCompositor will listen for incoming connections.
- `ip` - Only valid if `transport_protocol="udp"`. IP address where RTP packets should be sent to.
- `transportProtocol` - (**default=`"udp"`**) Transport layer protocol that will be used to send RTP packets.
  - `"udp"` - UDP protocol.
  - `"tcp_server"` - TCP protocol where LiveCompositor is the server side of the connection.
- `video` - Video stream configuration.
- `audio` - Audio stream configuration.

### `Outputs.RtpVideoOptions` 

```typescript
import React from "react"
import { Outputs } from "live-compositor"

type RtpVideoOptions = {
  resolution: {
    width: number;
    height: number;
  };
  sendEosWhen?: Outputs.OutputEndCondition;
  encoder: Outputs.RtpVideoEncoderOptions;
  root: React.ReactElement;
}
```

#### Properties
- `resolution` - Output resolution in pixels.
- `sendEosWhen` - Defines when output stream should end if some of the input streams are finished. If output includes both audio and video streams, then EOS needs to be sent on both.
- `encoder` - Video encoder options.
- `root` - Root of a component tree/scene that should be rendered for the output.

### `Outputs.RtpVideoEncoderOptions` 

```typescript
type RtpVideoEncoderOptions = 
  | {
      type: "ffmpeg_h264";
      preset: 
        | "ultrafast"
        | "superfast"
        | "veryfast"
        | "faster"
        | "fast"
        | "medium"
        | "slow"
        | "slower"
        | "veryslow"
        | "placebo";
      ffmpegOptions?: Map<string, string>;
    }
```

#### Properties (`type: "ffmpeg_h264"`)
- `preset` - (**default=`"fast"`**) Preset for an encoder. See `FFmpeg` [docs](https://trac.ffmpeg.org/wiki/Encode/H.264#Preset) to learn more.
- `ffmpegOptions` - Raw FFmpeg encoder options. See [docs](https://ffmpeg.org/ffmpeg-codecs.html) for more.

### `Outputs.RtpAudioOptions`

```typescript
import { Outputs } from "live-compositor"

type RtpAudioOptions = {
  mixingStrategy?: "sum_clip" | "sum_scale";
  sendEosWhen?: Outputs.OutputEndCondition;
  encoder: Outputs.RtpAudioEncoderOptions;
  initial?: {
    inputs: Outputs.InputAudio[];
  };
}
```

#### Properties
- `mixingStrategy` - (**default="sum_clip"**) Specifies how audio should be mixed.
  - `"sum_clip"` - Firstly, input samples are summed. If the result is outside the i16 PCM range, it gets clipped.
  - `"sum_scale"` - Firstly, input samples are summed. If the result is outside the i16 PCM range,
    nearby summed samples are scaled down by factor, such that the summed wave is in the i16 PCM range.
- `sendEosWhen` - Condition for termination of output stream based on the input streams states.
- `encoder` - Audio encoder options.
- `initial` - Initial audio mixer configuration for output.

### `Outputs.RtpAudioEncoderOptions`

```typescript
type RtpAudioEncoderOptions = 
  | {
      type: "opus";
      channels: "mono" | "stereo";
      preset?: "quality" | "voip" | "lowest_latency";
    }
```

#### Properties
- `channels` - Specifies channels configuration.
  - `"mono"` - Mono audio (single channel).
  - `"stereo"` - Stereo audio (two channels).
- `preset` - (**default="voip"**) Specifies preset for audio output encoder.
  - `"quality"` - Best for broadcast/high-fidelity application where the decoded audio
    should be as close as possible to the input.
  - `"voip"` - Best for most VoIP/videoconference applications where listening quality
    and intelligibility matter most.
  - `"lowest_latency"` - Only use when lowest-achievable latency is what matters most.

### `Outputs.InputAudio`

```typescript
type InputAudio = {
  inputId: string;
  volume?: number;
}
```

#### Properties
- `volume` - (**default=`1.0`**) float in `[0, 1]` range representing input volume

### `Outputs.OutputEndCondition`

```typescript
type OutputEndCondition = 
  | { anyOf: string[]; }
  | { allOf: string[]; }
  | { anyInput: boolean; }
  | { allInputs: boolean; };
```

This type defines when end of an input stream should trigger end of the output stream. Only one of those fields can be set at the time.
Unless specified otherwise the input stream is considered finished/ended when:
- TCP connection was dropped/closed.
- RTCP Goodbye packet (`BYE`) was received.
- Mp4 track has ended.
- Input was unregistered already (or never registered).

#### Properties
- `anyOf` - Terminate output stream if any of the input streams from the list are finished.
- `allOf` - Terminate output stream if all the input streams from the list are finished.
- `anyInput` - Terminate output stream if any of the input streams ends. This includes streams added after the output was registered. In particular, output stream will **not be** terminated if no inputs were ever connected.
- `allInputs` - Terminate output stream if all the input streams finish. In particular, output stream will **be** terminated if no inputs were ever connected.


