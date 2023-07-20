# Passing variables to the shader
We need to be able to pass an arbitrary number of textures and parameters to the custom shaders. For parameters, we need to be able to pass some "header parameters" -- constants used in all videos, as well as some per-video data.

### Possible solutions
Binding runtime-sized texture arrays works on every backend (except wasm, but it doesn't matter):
```wgsl
@group(0) @binding(0) var textures: texture_2d_array<f32>;
```

Binding const-sized uniform/storage buffers works everywhere too. This would work approximately like this in the shaders (ofc the len can be defined in a different way, e.g. as the length of a runtime-sized texture array):
```wgsl
struct MyList {
	// you declare an array with a constant, large size
	per_video_data: array<MyStruct, 64>
	// and then in some way specify how much of this buffer is filled
	per_video_data_len: u32
}

@group(0) @binding(0) var<uniform> per_video_data: MyList
```

Binding runtime-sized storage buffers only works on DX12 and Vulkan. Using this would look like this:
```wgsl
// the array is dynamically sized. 
// you can get the length using arrayLength(per_video_data)
@group(0) @binding(0) var<storage> per_video_data: array<MyStruct>
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
Use runtime-sized texture arrays, since it works everywhere, use const-sized uniforms, since not being compatible with Metal would harm development.

We should specify a binding in a bind group (e.g. binding 0 in group 1), where the uniform buffer will be attached in every shader.

## Potential Problems
- We need to be able to allocate an appropriately sized buffer for the parameters. This means that every node has to have it's own buffer. This is not a big problem, since the buffers are small, especially compared to textures.
- Careful error handling is necessary, since if the user gives us a buffer that is too short or misaligned the `wgpu`'s default reaction is to panic the process (there are mechanisms that change this behavior -- error scopes, we just need to remember to use them).
- It's becoming more and more apparent that we will need to expose some pipeline configuration options to the shader defining API, so that shaders can define the background color, or the formula used for alpha blending etc.
- We need to decide whether we want to use storage or uniform buffers for the parameters. Uniform buffers are faster, but limited in size to 64 KiB. Storage buffers are a bit slower, but support a much more generous 2 GiB of max size. Changing this later would be a breaking change, since the custom shaders would need to replace `var<uniform>` with `var<storage>` in their shader code. Do we want to allow the users to choose which kind of buffer they want to use?
- We need to remember that not all shaders need parameters.
