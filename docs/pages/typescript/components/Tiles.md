---
sidebar_position: 4
---
import AbsolutePositionDefinition from "@site/pages/common/absolute-position.md"
import TilesExample from "./tile_transition.webp"

# Tiles

`Tiles` is a layout component that places all the child components next to each other while maximizing the use of available space. The component divides its area into multiple rectangles/tiles, one for each child component. All of those rectangles are the same size and do not overlap over each other.

<div style={{textAlign: 'center'}}>
    <img src={TilesExample} style={{width: 600 }} />
</div>

### Absolute positioning

<AbsolutePositionDefinition />

- `Tiles` **does not** support absolute positioning for its child components. All children will still be rendered, but all fields like `top`, `left`, `right`, `bottom`, and `rotation` will be ignored.
- `Tiles` **can not** be absolutely positioned relative to its parent.

### Static positioning

The component calculates the number of rows and columns that children should be divided into. The result is based on:
- The size of the `Tiles` component.
- Aspect ratio of a single tile (`tile_aspect_ratio` field).
- Number of children components.

An optimal number of rows and columns should result in a layout that covers the biggest part of its area. Child components are placed based on their order, from left to right, and row-by-row from top to bottom.

When placing a child component inside a tile, the component might change its size.
- Non-layout component scales proportionally to fit inside the parent. If the aspect ratios of a child and its parent do not match, then the component will be centered vertically or horizontally.
- Layout component takes the `width` and `height` of a tile. It ignores its own `width`/`height` fields if they are defined.

### Transitions

The `Tiles` component does not support size transitions in the same way as `View` or `Rescaler` do. If you want to achieve that effect, you can wrap a `Tiles` component inside a `View` and define a transition on `View`.

Currently, supported transitions:
- Adding a new component. When a component is added, all of the existing components move to their new location within `transition.duration_ms` time. At the end of a transition, the new child component shows up without an animation.
- Removing an existing component. When a component is removed, a tile with that item disappears immediately without any animation, and the remaining elements move to their new location within `transition.duration_ms`.
- Changing the order of child components.


Adding/removing/changing the order of components can only be properly defined if there is a way to identify child components. We need to know if a specific child in a scene update should be treated as the same item as a child from a previous scene. Currently, identity of a child component is resolved in the following way:
- If a child component has an `"id"` defined, then this is its primary way of identification.
- If a child component does not have an `"id"` defined, then it's identified by order inside the `children` list, while only counting components without an `"id"`. For example:
  - A component without an `"id"` is 1st child in the old scene. After an update, the 1st component has an `"id"`, but the 2nd does not. In this situation, 1st component in the old scene and 2nd in the new one are considered to be the same component. It's the same because 2nd component in a new scene is still 1st if you only count components without an id.
  - There are two components without any `"id"` in the old scene. After an update, they switched places (still without any `"id"`). In that case, there would be no transition. Identification is based on the child components order, so from the `Tiles` component perspective only the content of those children has changed.

## TilesProps

```typescript
type TilesProps = {
  id?: string;
  children?: ReactElement[];
  width?: number;
  height?: number;
  backgroundColor?: string;
  tileAspectRatio?: string;
  margin?: number;
  padding?: number;
  horizontalAlign?: "left" | "right" | "justified" | "center";
  verticalAlign?: "top" | "center" | "bottom" | "justified";
  transition?: Transition;
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
- `backgroundColor` - (**default=`"#00000000"`**) Background color in a `#RRGGBBAA` or `#RRGGBB` format.
- `tileAspectRatio` - (**default=`"16:9"`**) Aspect ratio of a tile in `"W:H"` format, where W and H are integers.
- `margin` - (**default=`0`**) Margin of each tile in pixels.
- `padding` - (**default=`0`**) Padding on each tile in pixels.
- `horizontalAlign` - (**default=`"center"`**) Horizontal alignment of tiles.
- `verticalAlign` - (**default=`"center"`**) Vertical alignment of tiles.
- `transition` - Defines how this component will behave during a scene update. This will only have an
  effect if the previous scene already contained a `Tiles` component with the same id.

## Transition

```typescript
type Transition = {
  durationMs: number;
  easingFunction?: EasingFunction;
}
```

- `durationMs` - Duration of a transition in milliseconds.
- `easingFunction` - (**default=`"linear"`**) Easing function to be used for the transition.

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
