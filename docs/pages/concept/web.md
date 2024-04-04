# Web Renderer

## Overview

Web rendering is an experimental feature that lets you render websites.
Furthermore, you can place other components on the website. We refer to this process as embedding.

Make sure you have a compositor version built with web renderer support. The web renderer introduces additional dependencies and significantly increases the size of the compositor binaries. To minimize that impact, we are supporting two versions of the compositor, one with web renderer support and one without it.

:::tip
You can view a working example [here](https://github.com/membraneframework/video_compositor/blob/master/examples/web_view.rs)
:::

## Embedding components

Embedding is a process of displaying child components on a website. You can define the child components in the `children` field of the web view.
The child component IDs have to correspond to the IDs of HTML elements.
The web renderer embeds the children's frames in the specified HTML elements.

### Embedding methods

There are 3 embedding methods available:

- `chromium_embedding` - Frames produced by child components are passed directly to a chromium instance and displayed on an HTML canvas. Passing frames to the chromium instance introduces one more copy operation on each input frame, which may cause performance problems for a large number of inputs. The HTML elements used for embedding have to be canvases.
- `native_embedding_over_content` - Renders frames produced by child components on top of the website's content.
- `native_embedding_under_content` - Renders frames produced by child components below the website's content. The website needs to have a transparent background. Otherwise, it will cover the frames underneath it.

`native_embedding_over_content` is the default embedding method.
You can change it in the [register renderer request](../api/routes.md#register-web-renderer-instance). For example:

```typescript
{
    "type": "register",
    "entity_type": "web_renderer",
    "instance_id": "example_website",
    "url": "https://example.com",
    "resolution": {
        "width": 1920,
        "height": 1080
    },
    // highlight-next-line
    "embedding_method": "chromium_embedding"
}
```

## Example usage

Firstly, the web renderer instance has to be registered:

```typescript
{
    "type": "register",
    "entity_type": "web_renderer",
    "instance_id": "example_website",
    "url": "https://example.com",
    "resolution": {
        "width": 1920,
        "height": 1080
    },
    "embedding_method": "native_embedding_over_content"
}
```

- `instance_id` - unique renderer identifier. After the registration, the [web view](../api/components/WebView.md) component references the web renderer using this identifier.
- `url` - website url. All URL protocols supported by Chromium can be used here.

We can define a scene with a web view component that refers to the previously registered renderer instance using `instance_id` field:

```typescript
{
    "type": "update_output",
    "outputs": [
        {
            "output_id": "output_1",
            "scene": {
                "id": "embed_input_on_website",
                "type": "web_view",
                "instance_id": "example_website",
            }
        }
    ]
}
```

`instance_id` - the ID of previously registered web renderer.

:::warning
Only one web view component can use a specific web renderer instance at the same time.
:::

---

The above request defines a simple scene which displays a website.
Now, we can modify that request and embed an input stream into the website:

```typescript
{
    "type": "update_output",
    "outputs": [
        {
            "output_id": "output_1",
            "scene": {
                "id": "embed_input_on_website",
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
    ],
}
```

- `id` - the ID of an HTML element.

:::note
The input stream has to be registered beforehand.
:::

Web renderer places frames in HTML elements that are inside the website. Each HTML element must have an `id` attribute defined.
Here's an example website:

```html
<html>
    <body>
        <canvas id="my_video"></canvas>
    </body>
</html>
```

## Limitations

Underneath, the web renderer uses Chromium Embedded Framework. To render a website, we have to make a lot of copies, which can become a bottleneck. That is especially true for `chromium_embedding` since we have to copy frame data back and forth every frame.
