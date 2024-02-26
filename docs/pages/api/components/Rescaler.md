---
sidebar_position: 3
hide_table_of_contents: true
---
import Docs from "@site/pages/api/generated/component-Rescaler.md"
import TransitionDefinition from "@site/pages/common/transition.md"
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

<Docs />

<TransitionDefinition />
