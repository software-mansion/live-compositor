---
sidebar_position: 1
---

# InputStream

`InputStream` component represents a registered input.

:::note
To use this component, you need to first register the stream with matching `inputId` using [`LiveCompositor.registerInput`](../api.md#register-input) method.
:::

## Props

```typescript
type InputStreamProps = {
  id?: string;
  inputId: string;
  volume?: number;
  mute?: boolean;
}
```

- `id` - Id of a component. Defaults to value produced by `useId` hook.
- `inputId` - Id of an input. It identifies a stream registered using a [`LiveCompositor.registerInput`](../api.md#register-input) method.
- `volume` - (**default=`1`**) Audio volume represented by a number between 0 and 1.
- `mute` - (**default=`false`**) Mute audio.
