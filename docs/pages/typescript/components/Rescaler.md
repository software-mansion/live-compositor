---
sidebar_position: 3
---
import AbsolutePositionDefinition from "@site/pages/common/absolute-position.md"

# Rescaler

`Rescaler` is a layout component responsible for rescaling other components.

### Absolute positioning

<AbsolutePositionDefinition />

- `Rescaler` **does not** support absolute positioning for its child components. A child component will still be rendered, but all fields like `top`, `left`, `right`, `bottom`, and `rotation` will be ignored.
- `Rescaler` can be absolutely positioned relative to its parent, if the parent component supports it.

### Static positioning

`Rescaler` always have exactly one child that will be proportionally rescaled to match the parent.

### Transitions

On the scene update, a `Rescaler` component will animate between the original state and the new one if the `transition` field is defined. Both the original and the new scene need to define a component with the same `id`. Currently, only some of the fields support animated transitions:

- `width` / `height` - Only supported within the same positioning mode. If the positioning mode changes between the old scene and the new one, the transition will not work.
- `bottom` / `top` / `left` / `right` / `rotation` - Only supports transition when changing a value of the same field. If the old scene defines a `left` field and the new one does not, the transition will not work.

## RescalerProps

```typescript
type RescalerProps = {
  id?: string;
  children: ReactElement;
  mode?: "fit" | "fill";
  horizontalAlign?: "left" | "right" | "justified" | "center";
  verticalAlign?: "top" | "center" | "bottom" | "justified";
  width?: number;
  height?: number;
  top?: number;
  left?: number;
  bottom?: number;
  right?: number;
  rotation?: number;
  transition?: Transition;
}
```

- `id` - Id of a component. Defaults to value produced by `useId` hook.
- `children` - Rescaler accepts exactly one child component.
- `mode` - (**default=`"fit"`**) Resize mode:
  - `"fit"` - Resize the component proportionally, so one of the dimensions is the same as its parent,
    but it still fits inside it.
  - `"fill"` - Resize the component proportionally, so one of the dimensions is the same as its parent
    and the entire area of the parent is covered. Parts of a child that do not fit inside the parent are not rendered.
- `horizontalAlign` - (**default=`"center"`**) Horizontal alignment.
- `verticalAlign` - (**default=`"center"`**) Vertical alignment.
- `width` - Width of a component in pixels. Exact behavior might be different based on the parent
  component:
  - If the parent component is a layout, check sections "Absolute positioning" and "Static
  positioning" of that component.
  - If the parent component is not a layout, then this field is required.
- `height` - Height of a component in pixels. Exact behavior might be different based on the parent
  component:
  - If the parent component is a layout, check sections "Absolute positioning" and "Static
  positioning" of that component.
  - If the parent component is not a layout, then this field is required.
- `top` - Distance in pixels between this component's top edge and its parent's top edge.
  If this field is defined, then the component will ignore a layout defined by its parent.
- `left` - Distance in pixels between this component's left edge and its parent's left edge.
  If this field is defined, this element will be absolutely positioned, instead of being
  laid out by its parent.
- `bottom` - Distance in pixels between this component's bottom edge and its parent's bottom edge.
  If this field is defined, this element will be absolutely positioned, instead of being
  laid out by its parent.
- `right` - Distance in pixels between this component's right edge and its parent's right edge.
  If this field is defined, this element will be absolutely positioned, instead of being
  laid out by its parent.
- `rotation` - Rotation of a component in degrees. If this field is defined, this element will be
  absolutely positioned, instead of being laid out by its parent.
- `transition` - Defines how this component will behave during a scene update. This will only have an
  effect if the previous scene already contained a View component with the same id.

## Transition
```typescript
type Transition = {
  durationMs: number;
  easingFunction?: EasingFunction;
}
```

- `duration_ms` - Duration of a transition in milliseconds.
- `easing_function` - (**default=`"linear"`**) Easing function to be used for the transition.

## EasingFunction

```typescript
type EasingFunction = 
  | "linear"
  | "bounce"
  | {
      functionName: "cubic_bezier";
      points: [number, number, number, number];
    }
```
Easing functions are used to interpolate between two values over time.

Custom easing functions can be implemented with cubic Bézier.
The control points are defined with `points` field by providing four numerical values: `x1`, `y1`, `x2` and `y2`. The `x1` and `x2` values have to be in the range `[0; 1]`. The cubic Bézier result is clamped to the range `[0; 1]`.
You can find example control point configurations [here](https://easings.net/).
