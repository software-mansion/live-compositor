---
sidebar_position: 8
hide_table_of_contents: true
title: WebView
---

[<span class="badge badge--info">Required feature: web_renderer</span>](../../deployment/overview.md#web-renderer-support)

import Docs from "@site/pages/api/generated/component-WebView.md"

# WebView

`WebView` renders a website using Chromium engine embedded inside the compositor.

:::note
To use this component, you need to first register the web renderer instance with matching `instance_id` using [`register web renderer instance`](../routes#register-web-renderer-instance) request.
:::

:::warning
Only one component can use specific `instance_id` at the time.
:::

<Docs />
