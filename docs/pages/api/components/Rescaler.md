---
sidebar_position: 3
hide_table_of_contents: true
---
import Docs from "@site/pages/api/generated/component-Rescaler.md"
import AbsolutePositionDefinition from "@site/pages/common/absolute-position.md"

# Rescaler

`Rescaler` is a layout component responsible for rescaling other components.

### Absolute positioning

<AbsolutePositionDefinition />

- `Rescaler` **does not** support absolute positioning for its child components. A child component will still be rendered, but all fields like `top`, `left`, `right`, `bottom`, and `rotation` will be ignored.
- `Rescaler` can be absolutely positioned relative to its parent, if the parent component supports it.

### Static positioning

`Rescaler` always have exactly one child that will be proportionally rescaled to match the parent.

<Docs />
