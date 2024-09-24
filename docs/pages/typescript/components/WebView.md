---
sidebar_position: 8
title: WebView
---

[<span className="badge badge--info">Required feature: web_renderer</span>](../../deployment/overview.md#web-renderer-support)

# WebView

`WebView` renders a website using Chromium engine embedded inside the compositor.

:::note
To use this component, you need to first register the web renderer instance with matching `instanceId` using [`LiveCompositor.registerWebRenderer`](../api.md#register-web-renderer-instance) method.
:::

:::warning
Only one component can use specific `instanceId` at the time.
:::

## WebViewProps

```typescript
type WebView = {
  id?: string;
  children?: ReactElement[];
  instanceId: string;
}
```

WebView component renders a website using Chromium Embedded Framework (CEF).

- `id` - Id of a component. Defaults to value produced by `useId` hook.
- `children` - List of component's children.
- `instanceId` - Id of a web renderer instance. It identifies an instance registered using a [`LiveCompositor.registerWebRenderer`](../api.md#register-web-renderer-instance) request.
  
  :::warning
  You can only refer to specific instances in one Component at a time.
  :::
