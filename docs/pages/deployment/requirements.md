# Requirements

LiveCompositor imposes certain requirements both in the runtime and during a build. Specifics might differ depending on the way you use the compositor and features you are requiring.

## WebGPU features

If a feature is not supported then compositor server will fail on startup.

Always required:
- `TEXTURE_BINDING_ARRAY`
- `PUSH_CONSTANTS`

Enabled by default:
- `SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING`
- `UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING`

Those options are enabled by default, but can be disabled using [`LIVE_COMPOSITOR_REQUIRED_WGPU_FEATURES`](./configuration.md#live_compositor_required_wgpu_features) environment variable.

:::note
All of the above features should be available on almost any GPU. If you are getting an error
about unsupported features, then most likely, the issue is caused by missing or too old drivers.
:::

## Hardware requirements

Supported platforms:

- Linux - `x86_64`
  - It should work with any GPU from hardware perspective. Even though very unlikely, it is possible
    that some GPUs might not have drivers that implement all the required features.
- macOS - Apple Silicon

Other platforms are not regularly tested, but compositor should also work on:
- Linux - `aarch64` (Note that by default docker on Apple Silicon macOS is running `aarch64` Linux)
- macOS - `x86_64`

## Software requirements

### Dockerfile

Dockerfile defines all software requirements. Configurations provided in [the compositor repo](https://github.com/software-mansion/live-compositor/tree/master/build_tools/docker)
are written to work with both GPU and CPU based rendering. To use them in your own project, just copy
the Dockerfile and replace `COPY . /root/project` with an appropriate <nobr>`RUN git clone ...`</nobr> command.

If you want to use a different distro or different Ubuntu version then consult the [`building from source`](#building-from-source) section.

### Binaries from GitHub releases

For Linux:
- FFmpeg 6
- `glibc` 2.35 or higher (version used by Ubuntu 22.04)
- MESA (e.g. `mesa-vulkan-drivers` package on Ubuntu)
  - `23.2.1` or higher for CPU based rendering
  - For GPU based rendering the lowest version we tested was `22.0.1`, but older version might also work.

For macOS:
- FFmpeg 7 (this version will mostly follow a default version available in Homebrew)

:::note
Changing FFmpeg version on macOS might be troublesome, so we decided to produce binaries that will
be easiest to use for most users, even if this means updating FFmpeg version in a minor or patch release.
Changes like that might be considered unsafe, but in case of LiveCompositor macOS platform will be used
mostly for development, and it is highly unlikely to be used for a production deployment.
:::

### Building from source

- FFmpeg 6 or higher. Build time version has to be the same as runtime version.
- Rust toolchain.
- Following libraries (for build time you will need version with header files if your distro ships them separately):
  - FFmpeg dependencies: `libavcodec`, `libavformat`, `libavfilter`, `libavdevice`, `libavutil`, `libswscale`, `libswresample`
  - `libopus`
  - `libssl`
- cmake
- pkg-config

Linux specific (with a Vulkan backend):
- MESA
  - `23.2.1` or higher for CPU based rendering
  - For GPU based rendering the lowest version we tested was `22.0.1`, but older version might also work.
  - e.g. for Ubuntu
    - build time: `apt-get install libegl1-mesa-dev libgl1-mesa-dri libxcb-xfixes0-dev`
    - runtime: `apt-get install mesa-vulkan-drivers`

### Nix

LiveCompositor repository includes a nix flake, but because compositor is highly dependent on hardware and
available drivers this configuration might not be self-contained/portable as nix configuration are expected
to be in general.

This configuration is primary used for development, and it is not recommended unless you know what you are doing.
