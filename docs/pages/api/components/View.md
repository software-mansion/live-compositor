---
sidebar_position: 2
hide_table_of_contents: true
---
import Docs from "@site/pages/api/generated/component-View.md"
import AbsolutePositionDefinition from "@site/pages/common/absolute-position.md"

# View

`View` is the compositor's core layout mechanism. Its role is analogous to the
`<div>` tag in HTML. It provides a container with basic styling that can be further composed.

### Absolute positioning

<AbsolutePositionDefinition />

- `View` supports absolute positioning for its child components.
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

<Docs/>
