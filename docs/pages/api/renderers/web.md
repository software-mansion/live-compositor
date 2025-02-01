---
title: Web Renderer
---

import Docs from "@site/pages/api/generated/renderer-WebRenderer.md"

[<span class="badge badge--info">Required feature: web_renderer</span>](../../deployment/overview.md#web-renderer-support)

# Web Renderer

Represents an instance of a website opened with Chromium embedded inside the compositor. Used by a [`WebView` component](../components/WebView). Only one `WebView` component can use a specific instance at a time.
Before the web renderer can be used, you need to make sure that compositor with web rendering support is used.

<Docs />

## Environment variables

- `SMELTER_WEB_RENDERER_ENABLE` (default: `false`) - Enables web rendering capabilities.
- `SMELTER_WEB_RENDERER_GPU_ENABLE` (default: `true`) - If enabled, websites are rendered on GPU. Otherwise, software based rendering is used.

:::tip
Read more about environment variables [here](../../deployment/configuration.md#environment-variables)
:::
