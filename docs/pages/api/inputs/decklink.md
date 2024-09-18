---
title: DeckLink
hide_table_of_contents: true
---
import Docs from "@site/pages/api/generated/renderer-DeckLink.md"


<span class="badge badge--primary">Added in v0.3.0</span>
[<span class="badge badge--info">Required feature: decklink</span>](../../deployment/overview.md#decklink-support)

# DeckLink

An input type that allows consuming streams from Blackmagic DeckLink cards.

### Usage

To use DeckLink Input you must register it first. You can do it by sending a request like this:

<details>
    <summary>Example request</summary>

    ```http
    POST: /api/input/:input_id/register
    Content-Type: application/json
    ```

    ```js
    {
      "type": "decklink",
      "subdevice_index": 0,
      "display_name": "DeckLink Quad HDMI Recorder (3)",
      "persistent_id": "ffffffff",
      "enable_audio": false,
      "required": true,
    }
    ```
</details>

See [HTTP Routes](../routes.md#outputs-configuration) documentation to learn more about managing inputs.

<Docs />
