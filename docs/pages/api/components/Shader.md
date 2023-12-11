---
sidebar_position: 6
hide_table_of_contents: true
---

# Shader

`Shader` applies transformation defined via WGSL shader on its children. [Learn more.](../../concept/shaders)

:::note
To use this component, you need to first register the shader with matching `shader_id` using [`RegisterRenderer`](../routes#register-renderer) request.
:::

## Shader

```typescript
type Shader = {
  type: "shader";
  id: string;
  children?: Component[];
  shader_id: string;
  shader_param: ShaderParam;
  resolution: {
    width: u32,
    height: u32,
  }
}
```

#### Properties
- `id` - Id of a component.
- `children` - List of component's children.
- `shader_id` - Id of a shader. It identifies a shader registered using a [`RegisterRenderer`](../routes#register-renderer) request.
- `shader_param` - Object that will be serialized into a `struct` and passed inside the shader as:

  ```wgsl
  @group(1) @binding(0) var<uniform> user_defined_var: UserDefinedStruct;
  ```
  :::note
  This object's structure must match the structure defined in a shader source code. Currently, we do not handle memory layout automatically.
  To achieve the correct memory alignment, you might need to pad your data with additional fields. See [WGSL documentation](https://www.w3.org/TR/WGSL/#alignment-and-size) for more details.
  :::
- `resolution` - Resolution of a texture where shader will be executed.

## ShaderParam
```typescript
type ShaderParam =
  | { type: "f32"; value: f32 }
  | { type: "u32"; value: u32 }
  | { type: "i32"; value: i32 }
  | { type: "list"; value: ShaderParam[] }
  | { type: "struct"; value: ShaderParamStructField[] }

type ShaderParamStructField =
  | { field_name: string; type: "f32"; value: f32 }
  | { field_name: string; type: "u32"; value: u32 }
  | { field_name: string; type: "i32"; value: i32 }
  | { field_name: string; type: "list"; value: ShaderParam[] }
  | { field_name: string; type: "struct"; value: ShaderParamStructField[] }
```
