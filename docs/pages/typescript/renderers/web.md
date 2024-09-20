---
title: Web Renderer
---

[<span class="badge badge--info">Required feature: web_renderer</span>](../../deployment/overview.md#web-renderer-support)

# Web Renderer

Represents an instance of a website opened with Chromium embedded inside the compositor. Used by a [`WebView` component](../components/WebView). Only one `WebView` component can use a specific instance at a time.
Before the web renderer can be used, you need to make sure that compositor with web rendering support is used.

### `Renderer.RegisterWebRenderer`

```typescript
type RegisterWebRenderer = {
  url: string;
  resolution: {
    width: number;
    height: number;
  };
  embeddingMethod?: 
    | "chromium_embedding"
    | "native_embedding_over_content"
    | "native_embedding_under_content";
}
```

- `url` - Url of a website that you want to render.
- `resolution` - Resolution.
- `embeddingMethod` - Mechanism used to render input frames on the website.
  - `"chromium_embedding"` - Pass raw input frames as JS buffers so they can be rendered, for example, using a `<canvas>` component.
    :::warning
    This method might have a significant performance impact, especially for a large number of inputs.
    :::
  - `"native_embedding_over_content"` - Render a website without any inputs and overlay them over the website content.
  - `"native_embedding_under_content"` - Render a website without any inputs and overlay them under the website content.

## Environment variables

- `LIVE_COMPOSITOR_WEB_RENDERER_ENABLE` (default: `false`) - enables web rendering capabilities.
- `LIVE_COMPOSITOR_WEB_RENDERER_GPU_ENABLE` (default: `true`) - if enabled, websites are rendered on GPU. Otherwise, software based rendering is used.

:::tip
Read more about environment variables [here](../../deployment/configuration.md#environment-variables)
:::
