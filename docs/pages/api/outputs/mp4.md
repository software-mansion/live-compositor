---
title: MP4
hide_table_of_contents: true
---

import Docs from "@site/pages/api/generated/output-Mp4Output.md"

<span class="badge badge--primary">Added in v0.3.0</span>
<br />
<br />

# MP4

An output type that allows saving video and audio from the compositor to MP4 file.

### Usage

To use MP4 Output you need to register it first.

<details>
    <summary> Example register requests </summary>

    ```http
    POST: /api/output/:output_id/register
    Content-Type: application/json
    ```

    ```js
    {
      "type": "mp4",
      "path": "/path/to/file.mp4",
      "video": {
        "resolution": { "width": 1280, "height": 720 },
        "encoder": {
          "type": "ffmpeg_h264",
          "preset": "ultrafast"
        },
        "initial": {
          "root": {
            "type": "view",
            "background_color_rgba": "#4d4d4dff"
          }
        }
      },
      "audio": {
        "encoder": {
          "type": "aac",
          "channels": "stereo"
        },
        "initial": {
          "inputs": [{ "input_id": "input_1", "volume": 0.64 }]
        }
      }
    }
    ```
</details>

See [HTTP Routes](../routes.md#outputs-configuration) documentation to learn more about managing outputs.

<Docs />
