import Docs from "@site/pages/api/generated/renderer-RtpInputStream.md"

# RTP
An input type that allows streaming video and audio to the compositor over RTP.

### Usage

To use RTP Input you must register it first. You can do it by sending a request like this:

<details>
    <summary>Example request</summary>

    ```http
    POST: /api/input/:input_id/register
    Content-Type: application/json
    ```

    ```js
    {
    "type": "rtp_stream",
    "transport_protocol": "tcp_server",
    "port": 9001,
    "video": {
      "decoder": "ffmpeg_h264"
    },
    "audio": {
      "decoder": "opus"
    },
    "required": true,
    "offset_ms": 64
    }
    ```
</details>

See [HTTP Routes](../routes.md#outputs-configuration) documentation to learn more about managing inputs.
You can also check out [our guide](../../guides/deliver-input.md) to learn how to deliver streams after registering them.

<Docs />
