---
title: RTP
description: RTP Output
---
# RTP

An input type that allows streaming video and audio to the compositor over RTP.

### `Inputs.RegisterRtpInput` 

```typescript
import { Inputs } from "live-compositor"

type RegisterRtpInput = {
  port: string | u16;
  transportProtocol?: "udp" | "tcp_server";
  video?: Inputs.InputRtpVideoOptions;
  audio?: Inputs.InputRtpAudioOptions;
  required?: bool;
  offsetMs?: number;
}
```

Parameters for an input stream from RTP source.
At least one of `video` and `audio` has to be defined.

#### Properties
- `port` - UDP port or port range on which the compositor should listen for the stream.
- `transportProtocol` - Transport protocol.
  - `"udp"` - UDP protocol.
  - `"tcp_server"` - TCP protocol where LiveCompositor is the server side of the connection.
- `video` - Parameters of a video source included in the RTP stream.
- `audio` - Parameters of an audio source included in the RTP stream.
- `required` - (**default=`false`**) If input is required and the stream is not delivered
  on time, then LiveCompositor will delay producing output frames.
- `offsetMs` - Offset in milliseconds relative to the pipeline start (start request). If the offset is
  not defined then the stream will be synchronized based on the delivery time of the initial
  frames.

### `Inputs.InputRtpVideoOptions`

```typescript
type InputRtpVideoOptions = { decoder: "ffmpeg_h264"; }
```

### `Inputs.InputRtpAudioOptions` 

```typescript
type InputRtpAudioOptions = 
  | {
      decoder: "opus";
      forwardErrorCorrection?: bool;
    }
  | {
      decoder: "aac";
      audioSpecificConfig: string;
      rtpMode?: "low_bitrate" | "high_bitrate";
    }
```

#### Properties (`type: "opus"`)

- `forwardErrorCorrection` - (**default=`false`**) Specifies whether the stream uses forward error correction.
  It's specific for Opus codec.
  For more information, check out [RFC](https://datatracker.ietf.org/doc/html/rfc6716#section-2.1.7).

#### Properties (`type: "aac"`)

- `audioSpecificConfig` - AudioSpecificConfig as described in MPEG-4 part 3, section 1.6.2.1
  The config should be encoded as described in [RFC 3640](https://datatracker.ietf.org/doc/html/rfc3640#section-4.1).
  
  The simplest way to obtain this value when using ffmpeg to stream to the compositor is
  to pass the additional `-sdp_file FILENAME` option to ffmpeg. This will cause it to
  write out an sdp file, which will contain this field. Programs which have the ability
  to stream AAC to the compositor should provide this information.
  
  In MP4 files, the ASC is embedded inside the esds box (note that it is not the whole
  box, only a part of it). This also applies to fragmented MP4s downloaded over HLS, if
  the playlist uses MP4s instead of MPEG Transport Streams
  
  In FLV files and the RTMP protocol, the ASC can be found in the `AACAUDIODATA` tag.
- `rtpMode` - (**default=`"high_bitrate"`**)
  Specifies the [RFC 3640 mode](https://datatracker.ietf.org/doc/html/rfc3640#section-3.3.1)
  that should be used when depacketizing this stream.
