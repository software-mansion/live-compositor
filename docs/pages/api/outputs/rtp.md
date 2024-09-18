---
title: RTP
hide_table_of_contents: true
---

import Docs from "@site/pages/api/generated/output-RtpOutputStream.md"

# RTP

An output type that allows streaming video and audio from the compositor over RTP.

## Usage

To use RTP Output you must register it first. You can do it by sending a request like this:

<details>
    <summary>Example request</summary>

    ```http
    POST: /api/output/:output_id/register
    Content-Type: application/json
    ```

    ```js
    {
      "type": "rtp_stream",
      "transport_protocol": "tcp_server",
      "port": 9003,
      "video": {
        "resolution": { "width": 1280, "height": 720 },
        "encoder": {
          "type": "ffmpeg_h264",
          "preset": "ultrafast",
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
          "type": "opus",
          "channels": "mono",
        },
        "initial": {
          "inputs": [{ "input_id": "input_1" }]
        }
      }
    }
    ```
</details>

See [HTTP Routes](../routes.md#outputs-configuration) documentation to learn more about managing outputs.
You can also check out [our guide](../../guides/receive-output.md) to learn how to receive streams after registering them.

<Docs />
