# Shader parameter format

We need to transfer shader parameters in JSON. This document defines the transfer format.

## Format definition

The full format can be generated with the following rust type definitions and serde macros:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case", content = "value")]
pub enum ShaderParams {
    F32(f32),
    U32(u32),
    I32(i32),
    List(Vec<ShaderParams>),
    Struct(Vec<ShaderParamStructField>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShaderParamStructField {
    pub field_name: String,
    #[serde(flatten)]
    pub value: ShaderParams,
}
```

Additionally:

- all `wgsl` `vecN` types are represented as lists
- all `wgsl` `matNxM` types are represented as lists of lists

## Format demonstration

Let's look at a single type that we would like to represent. This is what it looks like in wgsl:

```wgsl
struct MyStruct {
    value_a: vec4<f32>,
    value_b: u32
}
```

This is what it would look like in rust:

```rust
#[repr(C)]
struct MyStruct {
    value_a: [f32; 4],
    value_b: u32,
}
```

This struct looks like this when encoded in the proposed format:

```json
{
    "type": "struct",
    "value": [
        {
            "field_name": "value_a",
            "type": "list",
            "value": [
                {
                    "type": "f32",
                    "value": 0.0
                },

                {
                    "type": "f32",
                    "value": 0.0
                },
                
                {
                    "type": "f32",
                    "value": 0.0
                },
                
                {
                    "type": "f32",
                    "value": 0.0
                }
            ]
        },

        {
            "field_name": "value_b"
            "type": "u32",
            "value": 42
        }
    ]
}
```
