# Shader

```typescript
Shader {
  type: "shader",

  node_id: NodeId,
  input_pads: Array<NodeId>,
  resolution: Resoltion,
  fallback_id?: NodeId,

  shader_id: string,
  shader_params?: Array<ShaderParam>,
}

ShaderParam = 
  | { type: "f32", value: number }
  | { type: "u32", value: number }
  | { type: "i32", value: number }
  | { type: "list", value: Array<ShaderParam> }
  | { type: "struct", value: Array<ShaderParam & { field_name: string }> }

```

See [shader documentation](https://github.com/membraneframework/video_compositor/wiki/Shader) to learn more.

- `input_pads` - List of inputs that will be available in the shader. Inside the shader source, those inputs will be available as textures. The number of textures needs to be constant, so if you have less than 16 inputs, small 1x1 empty textures are passed instead. Example shader code:

  ```wgsl
  @group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
  ```


- `shader_id` - Id of a previously registered [shader](https://github.com/membraneframework/video_compositor/wiki/Api-%E2%80%90-renderers#shader).
- `shader_params` - Parameters passed to the shader. This object needs to match the structure of a type defined in shader sources. Example shader code:

  ```wgsl
  struct ExampleUserDefinedParams {
      example_value: f32,
  }
  @group(1) @binding(0) var<uniform> example_user_provided_params: ExampleUserDefinedParams;
  ```
