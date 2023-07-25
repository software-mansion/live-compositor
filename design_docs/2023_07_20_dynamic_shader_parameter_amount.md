# Passing variables to the shader

We need to be able to pass an arbitrary number of textures and parameters to the custom shaders. For parameters, we need to be able to pass some "global parameters" -- constants used in all videos, as well as some per-video data.

## Possible solutions

### Textures

Unfortunately, it turns out that binding texture arrays is only possible when either

- Textures have the same resolution

or

- The shader always accepts a constant number of textures

This means we need to always bind a constant number of textures. If user's shader needs less textures than the length of the array, we can just fill the rest of it with references to a 1x1 texture which shouldn't harm performance in any way.

Obviously we also need to pass the amount of textures that are actually present in the array. We should use a push constant for this. Push constants are a very efficient mechanism for sending a very small amount of parameters to the shader.

Bringing it all together, the structure of variables declared in the user's shader could look like this:

```wgsl
@group(0) @binding(0) textures: binding_array<texture_2d<f32>, 16>;
var<push_constant> textures_len: u32;
```

### Shader parameters

Binding const-sized uniform/storage buffers works on every backend. This would work approximately like this in the shaders (ofc the len can be defined in a different way, e.g. as the length of the texture array):

```wgsl
struct MyList {
    // you declare an array with a constant, large size
    per_video_data: array<MyStruct, 64>
    // and then in some way specify how much of this buffer is filled
    per_video_data_len: u32
}

@group(1) @binding(1) var<uniform> per_video_data: MyList
```

We could also use runtime-sized storage buffers, which don't need a constant size (their size is defined at runtime). Unfortunately, binding runtime-sized storage buffers only works on DX12 and Vulkan. Using it would look like this:

```wgsl
// the array is dynamically sized. 
// you can get the length using arrayLength(per_video_data)
@group(1) @binding(1) var<storage> per_video_data: array<MyStruct>
```

Using this solution it's also possible to make one storage buffer that has some headers and then the dynamically sized per-video data (a runtime-sized array can be the last element of a struct, like in `C`):

```wgsl
struct MyStruct {
    some_param: u32,
    some_other_param: f32,
    per_video_data: array<MyVideoData>
}
```

This is also (sort of) possible using the first solution with a const limit on the video amount:

```wgsl
struct MyStruct {
    some_param: u32,
    some_other_param: f32,
    per_video_data: array<MyVideoData, 64>
}
```

### Proposed solution

Use the constant-sized texture arrays. Use const-sized uniforms, since not being compatible with Metal would harm development.

This document proposes the following bind groups structure:

```wgsl
// The textures rendered by previous nodes
@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;

// The amount of textures
var<push_constant> textures_len: u32

// The buffers defined by the API user (custom for every shader)
@group(1) @binding(0) var<uniform> user_defined_buffers: UserDefinedStructs;

// The buffers provided by the compositor (containing time information, output resolution etc.)
@group(1) @binding(1) var<uniform> compositor_buffers: CompositorBuffers;

// A sampler for sampling textures
@group(2) @binding(0) var sampler_: sampler
```

## Potential Problems

- We need to be able to allocate an appropriately sized buffer for the parameters. This means that every node has to have it's own buffer. This is not a big problem, since the buffers are small, especially compared to textures.
- Careful error handling is necessary, since if the user gives us a buffer that is too short or misaligned the `wgpu`'s default reaction is to panic the process (there are mechanisms that change this behavior -- error scopes, we just need to remember to use them).
- It's becoming more and more apparent that we will need to expose some pipeline configuration options to the shader defining API, so that shaders can define the background color, or the formula used for alpha blending etc.
- We need to decide whether we want to use storage or uniform buffers for the parameters. Uniform buffers are faster, but limited in size to 64 KiB. Storage buffers are a bit slower, but support a much more generous 2 GiB of max size. Changing this later would be a breaking change, since the custom shaders would need to replace `var<uniform>` with `var<storage>` in their shader code. Do we want to allow the users to choose which kind of buffer they want to use?
  - Decision: use uniforms for now, add support for storage buffers later
- We need to remember that not all shaders need parameters.
- We need to consider adding support for creating an additional type of shaders: ones that always take the same amount of inputs
