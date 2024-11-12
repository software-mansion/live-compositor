---
title: MP4
hide_table_of_contents: true
---

import Docs from "@site/pages/api/generated/renderer-Mp4Input.md"

# MP4
An input type that allows the compositor to read static MP4 files.

Mp4 files can contain video and audio tracks encoded with various codecs.
This input type supports mp4 video tracks encoded with h264 and audio tracks encoded with AAC.

If the file contains multiple video or audio tracks, the first audio track and the first video track will be used and the other ones will be ignored.

### Usage

To use MP4 Input you must register it first. You can do it by sending a request like this:

<details>
    <summary>Example request</summary>

    ```http
    POST: /api/input/:input_id/register
    Content-Type: application/json
    ```

    ```js
    {
      "type": "mp4"
      "url": "https://url.to.file.mp4"
    }
    ```
</details>

See [HTTP Routes](../routes.md#outputs-configuration) documentation to learn more about managing inputs.

<Docs />
