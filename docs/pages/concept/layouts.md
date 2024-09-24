# Layouts

Layout components define the size, position, and simple styling of other components.

Currently, we support the following layout components:
- `View` - ([`TypeScript`](../typescript/components/View.md), [`HTTP`](../api/components/View.md))
- `Tiles` - ([`TypeScript`](../typescript/components/Tiles.md), [`HTTP`](../api/components/Tiles.md))
- `Rescaler` - ([`TypeScript`](../typescript/components/Rescaler.md), [`HTTP`](../api/components/Rescaler.md))

## Common properties

Most layout components share a set of common properties.

- `width` - Width of a component in pixels.
- `height` - Height of a component in pixels.

### Absolute positioning properties

When a component is positioned absolutely, it will ignore the normal layout of its parent.

Common properties that imply the component will be absolutely positioned:

- `top` - Distance in pixels between this component's top edge and its parent's top edge.
- `bottom` - Distance in pixels between this component's bottom edge and its parent's bottom edge.
- `left` - Distance in pixels between this component's left edge and its parent's left edge.
- `right` - Distance in pixels between this component's right edge and its parent's right edge.
- `rotation` - Rotation in degrees.

:::warning
Not all components support everything listed above. Consult the API reference for each component to verify it.
:::

### Size

The size of a layout component is defined by its parent:
- If a layout component is a root in a component tree, then its size is based on the declared resolution of an output stream.
- If a layout component is a child of a non-layout component, then it has to have its size defined, usually via the `width`/`height` fields.
- If a layout component is a child of another layout component, then, unless explicitly defined, its size will be based on the area defined by its parent. For example:
  - For the `Tiles` component, it will be an area of a single tile.
  - For the `View` component, it will be an area calculated based on the sizes of other sibling components.


