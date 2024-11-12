---
sidebar_position: 7
---
# Image

A component for rendering images.

:::note
To use this component, you need to first register the image with matching `imageId` using [`LiveCompositor.registerImage`](../instance.md#register-image) request.
:::

## ImageProps

```typescript
type ImageProps = {
  id?: string;
  imageId: string;
}

```

#### Properties
- `id` - Id of a component. Defaults to value produced by `useId` hook.
- `imageId` - Id of an image. It identifies an image registered using a [`LiveCompositor.registerImage`](../instance.md#register-image) method.
