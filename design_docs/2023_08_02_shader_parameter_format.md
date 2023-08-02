# Shader parameter format

What kind of format should we use for transferring shader parameters in json? I've been thinking about two designs.

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

## First idea

We only transport an array of primitives, as they are laid out in the type after stripping out all structs, vecs, matrices and arrays:

```json
[
    {
        "type": "f32",
        "val": 0.0,
    },

    {
        "type": "f32",
        "val": 0.0,
    },

    {
        "type": "f32",
        "val": 0.0,
    },

    {
        "type": "f32",
        "val": 0.0,
    },

    {
        "type": "u32",
        "val": 42,
    },
]
```

## Second idea

We include information about the structure of the data:

```json
{
    "type": "struct",
    "val": [
        {
            "type": "list",
            "val": [
                {
                    "type": "f32",
                    "val": 0.0
                },

                {
                    "type": "f32",
                    "val": 0.0
                },
                
                {
                    "type": "f32",
                    "val": 0.0
                },
                
                {
                    "type": "f32",
                    "val": 0.0
                }
            ]
        },

        {
            "type": "u32",
            "val": 42
        }
    ]
}
```

There are varying degrees of the strictness of this approach, in this example I chose to treat `vec4` as a `"list"`. `vec2`, `vec3` and `array` would probably all be considered a `"list"`, but in a more strict version of this approach all of them could be separate types. The more annoying thing about this approach is the necessary validation. We should validate that all values in a list type are of the same type. Using more strict typing we would have to enforce that the `vec`s and matrices all have correct amounts of values and don't include non-primitive types.

What format should we choose?
