# Component

A component is a basic block used to define how video streams are composed.

## Layout Components

Layout component is a type of component responsible for defining the size and position of other components.

Currently, we support the following layout components:
- [View](../api/components/View)
- [Tiles](../api/components/Tiles)
- [Rescaler](../api/components/Rescaler)

Learn more about layouts [here](./layouts).

## Non-layout components

Non-layout components have their unique behaviors. In most cases, they do not support or interact with mechanisms introduced by layouts. Sometimes, they even override the behavior of other components.

For example, if you create a `Shader` component with a `View` component as its child, the properties like `width`, `top`, `rotation` ..., will be ignored. A `Shader` component, when rendering, receives all its children as GPU textures. It will just execute whatever the user-provided shader source implements without applying any layout properties that component might have.

## Scene

Component tree that represents what will be rendered for a specific output.

Example scene:
```typescript
{
    "type": "view",
    "background_color_rgba": "#0000FFFF"
    "children": [
        {
            "type": "input-stream",
            "input_id": "example_input_1",
        }
    ]
}
```

In the example above, we define a scene where an input stream `example_input_1` is rendered inside a [`View` component](../api/components/View). You can configure that scene for a specific output in the [`RegisterOutputStream` request](../api/routes#register-output-stream) using `initial_scene` field or in the [`UpdateScene` request](../api/routes#update-scene).

:::note
You need to register `"example_input_1"` before using it in the scene definition.
:::

### Renderers

Renderers are entities capable of producing frames (in some cases based on some provided input). The renderer could be a WGSL shader, web renderer instance, or an image. They are not directly part of the scene definition. Instead, components are using them as part of their internal implementation.

For example:
- [The `Shader` component](../api/components/Shader) has a field `shader_id` that identifies a [`Shader` renderer](../api/renderers/Shader).
- [The `Image` component](../api/components/Image) has a field `image_id` that identifies an [`Image` renderer](../api/renderers/Image).

Every renderer, except [`WebRenderer`](../api/renderers/web), can be used in multiple components. For example, you can create a single `Shader` renderer that applies some effect and use that `shader_id` in multiple `Shader` components.


