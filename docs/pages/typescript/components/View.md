---
sidebar_position: 2
---
import AbsolutePositionDefinition from "@site/pages/common/absolute-position.md"

# View

`View` is the compositor's core layout mechanism. Its role is analogous to the
`<div>` tag in HTML. It provides a container with basic styling that can be further composed.

### Absolute positioning

<AbsolutePositionDefinition />

- `View` supports absolute positioning for its child components. If not provided explicitly, an absolutely positioned child will inherit `"width"` and `"height"` from the parent.
- `View` can be absolutely positioned relative to its parent if the parent component supports it.

### Static positioning

When children of a `View` component have a static position, they are placed next to each other.

#### For `direction=row`:

Children of a `View` component form a row, with items aligned to the top. The size of each child will be calculated in the following way:
- If the `width` or `height` of a child component is defined, then those values take priority.
- If the `height` is not defined, the component will have the same `height` as its parent.
- If the `width` is not defined, we calculate the sum `width` of all components with that value defined.
  - If it is larger than the parent's `width`, then the `width` of the rest of the components is zero.
  - If it is smaller than the parent's `width`, calculate the difference and divide the resulting value equally between all children with unknown widths.

#### For `direction=column`:

Analogous to the `direction=row` case, but children form a column instead, with items aligned to the left.

### Transitions

On the scene update, a `View` component will animate between the original state and the new one if the `transition` field is defined. Both the original and the new scene need to define a component with the same `id`. Currently, only some of the fields support animated transitions:

- `width` / `height` - Only supported within the same positioning mode. If the positioning mode changes between the old scene and the new one, the transition will not work.
- `bottom` / `top` / `left` / `right` / `rotation` - Only supports transition when changing a value of the same field. If the old scene defines a `left` field and the new one does not, the transition will not work.

## ViewProps

```typescript
type ViewProps = {
  id?: string;
  children?: ReactElement[];
  width?: number;
  height?: number;
  direction?: "row" | "column";
  top?: number;
  left?: number;
  bottom?: number;
  right?: number;
  rotation?: number;
  transition?: Transition;
  overflow?: "visible" | "hidden" | "fit";
  backgroundColor?: string;
}
```

- `id` - Id of a component. Defaults to value produced by `useId` hook.
- `children` - List of component's children.
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
- `direction` - (**default=`"row"`**) Direction defines how static children are positioned inside a View component.
  - `"row"` - Children positioned from left to right.
  - `"column"` - Children positioned from top to bottom.
- `top` - Distance in pixels between this component's top edge and its parent's top edge.
  If this field is defined, then the component will ignore a layout defined by its parent.
- `left` - Distance in pixels between this component's left edge and its parent's left edge.
  If this field is defined, this element will be absolutely positioned, instead of being
  laid out by its parent.
- `bottom` - Distance in pixels between the bottom edge of this component and the bottom edge of its parent.
  If this field is defined, this element will be absolutely positioned, instead of being
  laid out by its parent.
- `right` - Distance in pixels between this component's right edge and its parent's right edge.
  If this field is defined, this element will be absolutely positioned, instead of being
  laid out by its parent.
- `rotation` - Rotation of a component in degrees. If this field is defined, this element will be
  absolutely positioned, instead of being laid out by its parent.
- `transition` - Defines how this component will behave during a scene update. This will only have an
  effect if the previous scene already contained a View component with the same id.
- `overflow` - (**default=`"hidden"`**) Controls what happens to content that is too big to fit into an area.
  - `"visible"` - Render everything, including content that extends beyond their parent.
  - `"hidden"` - Render only parts of the children that are inside their parent area.
  - `"fit"` - If children components are too big to fit inside the parent, resize everything inside to fit.
    
    Components that have unknown sizes will be treated as if they had a size 0 when calculating
    scaling factor.
    
    :::warning
    This will resize everything inside, even absolutely positioned elements. For example, if
    you have an element in the bottom right corner and the content will be rescaled by a factor 0.5x,
    then that component will end up in the middle of its parent
    :::
- `backgroundColor` - (**default=`"#00000000"`**) Background color in a `"#RRGGBBAA"` or `#RRGGBB` format.

### Transition

```typescript
type Transition = {
  durationMs: f64;
  easingFunction?: EasingFunction;
}
```

- `durationMs` - Duration of a transition in milliseconds.
- `easingFunction` - (**default=`"linear"`**) Easing function to be used for the transition.

### EasingFunction

```typescript
type EasingFunction = 
  | "linear"
  | "bounce"
  | {
      functionName: "cubic_bezier";
      points: [f64, f64, f64, f64];
    }
```
Easing functions are used to interpolate between two values over time.

Custom easing functions can be implemented with cubic Bézier.
The control points are defined with `points` field by providing four numerical values: `x1`, `y1`, `x2` and `y2`. The `x1` and `x2` values have to be in the range `[0; 1]`. The cubic Bézier result is clamped to the range `[0; 1]`.
You can find example control point configurations [here](https://easings.net/).
