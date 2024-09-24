---
title: MP4
---
<span class="badge badge--primary">Added in v0.3.0</span>
<br />
<br />

# MP4

An output type that allows saving video and audio from the compositor to MP4 file.

### `Outputs.RegisterMp4Output` 

```typescript
import { Outputs } from "live-compositor"

type RegisterMp4Output = {
  serverPath: string;
  video?: Outputs.Mp4VideoOptions;
  audio?: Outputs.Mp4AudioOptions;
}
```

#### Properties

- `serverPath` - Path to the MP4 file (location on the server where LiveCompositor server is deployed).
- `video` - Video track configuration.
- `audio` - Audio track configuration.

### `Outputs.Mp4VideoOptions` 

```typescript
import React from 'react'
import { Outputs } from "live-compositor"

type Mp4VideoOptions = {
  resolution: {
    width: number;
    height: number;
  };
  sendEosWhen?: Outputs.OutputEndCondition;
  encoder: Outputs.Mp4VideoEncoderOptions;
  root: React.ReactElement;
}
```

#### Properties
- `resolution` - Output resolution in pixels.
- `sendEosWhen` - Defines when output stream should end if some of the input streams are finished. If output includes both audio and video streams, then EOS needs to be sent on both.
- `encoder` - Video encoder options.
- `root` - Root of a component tree/scene that should be rendered for the output.

### `Outputs.Mp4VideoEncoderOptions` 

```typescript
type Mp4VideoEncoderOptions = 
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

### `Outputs.Mp4AudioOptions`

```typescript
import { Outputs } from "live-compositor"

type Mp4AudioOptions = {
  mixingStrategy?: "sum_clip" | "sum_scale";
  sendEosWhen?: Outputs.OutputEndCondition;
  encoder: Outputs.Mp4AudioEncoderOptions;
  initial: {
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

### `Outputs.Mp4AudioEncoderOptions`

```typescript
type Mp4AudioEncoderOptions = {
  type: "aac";
  channels: "mono" | "stereo";
}
```

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
