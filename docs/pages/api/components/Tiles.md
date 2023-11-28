---
sidebar_position: 4
---
import Docs from "@site/pages/api/generated/component-Tiles.md"

# Tiles

`Tiles` is a layout component that places all the child components next to each other while maximizing the use of available space. The component divides its area into multiple rectangles/tiles, one for each child component. All of those rectangles are the same size and do not overlap over each other.

### Absolute positioning

- `Tiles` **does not** support absolute positioning for its child components. All children will still be rendered, but all fields like `top`, `left`, `right`, `bottom`, and `rotation` will be ignored.
- `Tiles` **can not** be absolutely positioned relative to it's parent.

### Static positioning

The component calculates the number of rows and columns that children should be divided into. The result is based on:
- The size of the `Tiles` component.
- Aspect ratio of a single tile (`tile_aspect_ratio` field).
- Number of children components.

An optimal number of rows and columns should result in a layout that covers the biggest part of its area. Children components are placed based on their order, from left to right, and row-by-row from top to bottom.

When placing a child component inside a tile, the component might change its size.
- Non-layout component scales proportionally to fit inside the parent. If the aspect ratios of a child and its parent do not match, then the component will be centered vertically or horizontally.
- Layout component takes the `width` and `height` of a tile. It ignores its own `with`/`height` fields if they are defined.

<Docs />
