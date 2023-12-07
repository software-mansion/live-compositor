---
sidebar_position: 3
---
import Docs from "@site/pages/api/generated/component-Rescaler.md"

# Rescaler

`Rescaler` is a layout component responsible for rescaling other components.

### Absolute positioning

A component is absolutely positioned if it defines fields like `top`, `left`, `right`, `bottom`, or `rotation`.
Those fields define the component's position relative to its parent. However, to respect those
values, the parent component has to be a layout component that supports absolute positioning.

- `Rescaler` **does not** support absolute positioning for its child components. All children will still be rendered, but all fields like `top`, `left`, `right`, `bottom`, and `rotation` will be ignored.
- `Rescaler` can be absolutely positioned relative to its parent, if the parent component supports it.

### Static positioning

`Rescaler` always have exactly one child that will be proportionally rescaled to match the parent.

<Docs />
