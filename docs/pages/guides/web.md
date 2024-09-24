import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

# Web Renderer

Web renderer allows you to capture that output of a browser and compose it with other streams.

## Overview

Web rendering is an experimental feature that lets you render websites.
Furthermore, you can place other components on the website. We refer to this process as embedding.

Make sure you have a compositor version built with web renderer support. The web renderer introduces additional dependencies and significantly increases the size of the compositor binaries. To minimize that impact, we are supporting two versions of the compositor, one with web renderer support and one without it.

## Embedding components

Embedding is a process of displaying child components on a website. You can define the child components in the `children` field of the web view.
The child component IDs have to correspond to the IDs of HTML elements.
The web renderer embeds the children's frames in the specified HTML elements.

### Embedding methods

There are 3 embedding methods available:

- `chromium_embedding` - Frames produced by child components are passed directly to a chromium instance and displayed on an HTML canvas. Passing frames to the chromium instance introduces one more copy operation on each input frame, which may cause performance problems for a large number of inputs. The HTML elements used for embedding have to be canvases.
- `native_embedding_over_content` - Renders frames produced by child components on top of the website's content.
- `native_embedding_under_content` - Renders frames produced by child components below the website's content. The website needs to have a transparent background. Otherwise, it will cover the frames underneath it.

You can define embedding method when registering a renderer. `native_embedding_over_content` is the default. For example:

<Tabs queryString="lang">
  <TabItem value="react" label="React">
    ```tsx
    await compositor.registerWebRenderer("example_website", {
      url: "https://example.com",
      resolution: {
        width: 1920,
        height: 1080
      },
      // highlight-next-line
      embeddingMethod: "chromium_embedding"
    })
    ```
  </TabItem>
  <TabItem value="http" label="HTTP">
    ```http
    POST: /api/web-renderer/example_website/register
    Content-Type: application/json

    {
      "url": "https://example.com",
      "resolution": {
        "width": 1920,
        "height": 1080
      },
      // highlight-next-line
      "embedding_method": "chromium_embedding"
    }
    ```
  </TabItem>
</Tabs>

## Example usage

Firstly, the web renderer instance has to be registered:

<Tabs queryString="lang">
  <TabItem value="react" label="React">
    ```tsx
    await compositor.registerWebRenderer("example_website", {
      url: "https://example.com",
      resolution: {
        width: 1920,
        height: 1080
      },
      embeddingMethod: "native_embedding_over_content"
    })
    ```

    - `instanceId` - unique renderer identifier. After the registration, the [`WebView`](../typescript/components/WebView.md) component references the web renderer using this identifier.
    - `url` - website url. All URL protocols supported by Chromium can be used here.

    After registration you can use it by adding `<WebView instanceId="example_website" />` in the scene.
  </TabItem>
  <TabItem value="http" label="HTTP">
    ```http
    POST: /api/web-renderer/example_website/register
    Content-Type: application/json

    {
      "url": "https://example.com",
      "resolution": {
        "width": 1920,
        "height": 1080
      },
      "embedding_method": "native_embedding_over_content"
    }
    ```

    - `instance_id` - unique renderer identifier. After the registration, the [`WebView`](../api/components/WebView.md) component references the web renderer using this identifier.
    - `url` - website url. All URL protocols supported by Chromium can be used here.

    We can define a scene with a web view component that refers to the previously registered renderer instance using `instance_id` field:



    ```http
    POST: /api/output/output_1/update
    Content-Type: application/json

    {
      "video": [
        "type": "web_view",
        "instance_id": "example_website",
      ]
    }
    ```

    `instance_id` - the ID of previously registered web renderer.
  </TabItem>
</Tabs>

:::warning
Only one web view component can use a specific web renderer instance at the time.
:::

---

The above request defines a simple scene which displays a website.
Now, we can modify that request and embed an input stream into the website:

<Tabs queryString="lang">
  <TabItem value="react" label="React">
    ```tsx
    function App() {
      return (
        <WebView instanceId="example_website">
          // highlight-start
          <InputStream id="my_video" inputId="input_1" />
          // highlight-end
        </WebView>
      )
    }
    ```
  </TabItem>
  <TabItem value="http" label="HTTP">
    ```http
    POST: /api/output/output_1/update
    Content-Type: application/json

    {
      "video": {
        "type": "web_view",
        "instance_id": "example_website",
        // highlight-start
        "children": [
          {
            "id": "my_video",
            "type": "input_stream",
            "input_id": "input_1",
          }
        ]
        // highlight-end
      }
    }
    ```
  </TabItem>
</Tabs>

- `id` - the ID of an HTML element.

:::note
The input stream has to be registered beforehand.
:::

Web renderer places frames in HTML elements that are inside the website. Each HTML element must
have an `id` attribute defined. Here's an example website:

```html
<html>
  <body>
    <canvas id="my_video"></canvas>
  </body>
</html>
```

## Limitations

Internally, the web renderer uses Chromium Embedded Framework. To render a website, we have to copy frames between CPU and GPU memory, which can become a bottleneck. That is especially true for `chromium_embedding` since we have to copy frames from all children components.
