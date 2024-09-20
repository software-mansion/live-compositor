---
sidebar_position: 6
---
# Shader

`Shader` applies transformation defined via WGSL shader on its children. [Learn more.](../../concept/shaders.md)

:::note
To use this component, you need to first register the shader with matching `shaderId` using [`LiveCompositor.registerShader`](../api.md#register-shader) method.
:::

## ShaderProps

```typescript
type ShaderProps = {
  id?: string;
  children?: ReactElement[];
  shaderId: string;
  shaderParam?: ShaderParam;
  resolution: {
    width: number;
    height: number;
  };
}
```

- `id` - Id of a component. Defaults to value produced by `useId` hook.
- `children` - List of component's children.
- `shaderId` - Id of a shader. It identifies a shader registered using a [`LiveCompositor.registerShader`](../api.md#register-shader) method.
- `shaderParam` - Object that will be serialized into a `struct` and passed inside the shader as:
  
  ```wgsl
  @group(1) @binding(0) var<uniform>
  ```
  :::note
  This object's structure must match the structure defined in a shader source code. Currently, we do not handle memory layout automatically.
  To achieve the correct memory alignment, you might need to pad your data with additional fields. See [WGSL documentation](https://www.w3.org/TR/WGSL/#alignment-and-size) for more details.
  :::
- `resolution` - Resolution of a texture where shader will be executed.

## ShaderParam

```typescript
type ShaderParam = 
  | { type: "f32"; value: number; }
  | { type: "u32"; value: number; }
  | { type: "i32"; value: number; }
  | { type: "list"; value: ShaderParam[]; }
  | {
      type: "struct";
      value: ShaderParamStructField[];
    }
```

## ShaderParamStructField

```typescript
type ShaderParamStructField = 
  | { fieldName: string; type: "f32"; value: number; }
  | { fieldName: string; type: "u32"; value: number; }
  | { fieldName: string; type: "i32"; value: number; }
  | {
      fieldName: string;
      type: "list";
      value: ShaderParam[];
    }
  | {
      fieldName: string;
      type: "struct";
      value: ShaderParamStructField[];
    }
```
