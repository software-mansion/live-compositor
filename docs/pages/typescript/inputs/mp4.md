---
title: MP4
description: MP4 Output
---

# MP4
An input type that allows the compositor to read static MP4 files.

Mp4 files can contain video and audio tracks encoded with various codecs.
This input type supports mp4 video tracks encoded with h264 and audio tracks encoded with AAC.

If the file contains multiple video or audio tracks, the first audio track and the first video track will be used and the other ones will be ignored.

## `Inputs.RegisterMp4Input`

```typescript
type Mp4Input = {
  url?: string;
  serverPath?: string;
  loop?: bool;
  required?: bool;
  offsetMs?: f64;
}
```

Input stream from MP4 file.
Exactly one of `url` and `path` has to be defined.

#### Properties
- `url` - URL of the MP4 file.
- `serverPath` - Path to the MP4 file (location on the server where LiveCompositor server is deployed).
- `loop` - (**default=`false`**) If input should be played in the loop. <span class="badge badge--primary">Added in v0.4.0</span>
- `required` - (**default=`false`**) If input is required and frames are not processed
  on time, then LiveCompositor will delay producing output frames.
- `offsetMs` - Offset in milliseconds relative to the pipeline start (start request). If offset is
  not defined then stream is synchronized based on the first frames delivery time.
