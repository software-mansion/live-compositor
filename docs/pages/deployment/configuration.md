# Configuration

## Environment variables

### `LIVE_COMPOSITOR_API_PORT`

API port. Defaults to 8081.

### `LIVE_COMPOSITOR_INSTANCE_ID`

ID that will be returned in `GET /status` request. Can be used to identify if we are connecting to the correct compositor instance.

### `LIVE_COMPOSITOR_OUTPUT_FRAMERATE`

Output framerate for all output streams. This value can be a number or string in the `NUM/DEN` format, where both `NUM` and `DEN` are unsigned integers. Defaults to `30`

### `LIVE_COMPOSITOR_MIXING_SAMPLE_RATE`

Sample rate used for mixing audio. This value has to be a number or string representing supported sample rate. Defaults to 48000.

Supported sample rates are: 8000, 12000, 16000, 24000, 44100, 48000

### `LIVE_COMPOSITOR_FORCE_GPU`

If enabled, GPU will be required for rendering. If only CPU based adapters will be found then process will exit with an error. Defaults to `false`.

### `LIVE_COMPOSITOR_STREAM_FALLBACK_TIMEOUT_MS`

A timeout that defines when the compositor should switch to fallback on the input stream that stopped sending frames. Defaults to 500.

### `LIVE_COMPOSITOR_LOGGER_LEVEL`

Logger level. Value can be defined as `error`/`warn`/`info`/`debug`/`trace`.

This value also supports syntax for more detailed configuration. See [`tracing-subscriber` crate documentation](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax) for more info.

Defaults to `info,wgpu_hal=warn,wgpu_core=warn`.

### `LIVE_COMPOSITOR_LOGGER_FORMAT`

Logger format. Supported options:

- `json`
- `compact`
- `pretty`

:::warning
This option does not apply to logs produced by `FFmpeg` or the embedded Chromium instance used for web rendering. Defaults to `json`.
:::

### `LIVE_COMPOSITOR_FFMPEG_LOGGER_LEVEL`

Minimal log level that should be logged. Supported options:

- `error` - equivalent to FFmpeg's `error, 16`
- `warn` - equivalent to FFmpeg's `warning, 24`
- `info` - equivalent to FFmpeg's `info, 32`
- `debug` - equivalent to FFmpeg's `debug, 48`

See `-loglevel` option in [FFmpeg documentation](https://ffmpeg.org/ffmpeg.html). Defaults to `warn`.

### `LIVE_COMPOSITOR_DOWNLOAD_DIR`

A path to a directory in which downloaded files will be stored. Uses the location provided by the OS if not set.

In this directory, an instance of the compositor will create a subdirectory named `live-compositor-<random number>`. Downloaded temporary files will be stored there.

### `LIVE_COMPOSITOR_WEB_RENDERER_ENABLE`

Enable web rendering capabilities. With this option disabled, you can not use [`WebView` components](../api/components/WebView) or register [`WebRenderer` instances](../api/renderers/web).

Defaults to `false`. Valid values: `true`, `false`, `1`, `0`.

### `LIVE_COMPOSITOR_WEB_RENDERER_GPU_ENABLE`

Enable GPU support inside the embedded Chromium instance.

Defaults to `true`. Valid values: `true`, `false`, `1`, `0`.

### `LIVE_COMPOSITOR_OFFLINE_PROCESSING_ENABLE`

If enabled, sets `LIVE_COMPOSITOR_AHEAD_OF_TIME_PROCESSING_ENABLE` and `LIVE_COMPOSITOR_NEVER_DROP_OUTPUT_FRAMES` options to `true`. If those values are also defined then they take priority over this value.

Defaults to `false`. Valid values: `true`, `false`, `1`, `0`.

### `LIVE_COMPOSITOR_AHEAD_OF_TIME_PROCESSING_ENABLE`

If enabled, the LiveCompositor server will try to generate output frames/samples ahead of time if all inputs are available.

When to enable this option:

- If you want to process input streams faster than in real time.

Defaults to `false`. Valid values: `true`, `false`, `1`, `0`.

### `LIVE_COMPOSITOR_NEVER_DROP_OUTPUT_FRAMES`

If enabled, the LiveCompositor server will not drop frames/samples from output stream even if rendering or encoding is not fast enough to process it in real time.

Defaults to `false`. Valid values: `true`, `false`, `1`, `0`.

### `LIVE_COMPOSITOR_RUN_LATE_SCHEDULED_EVENTS`

Parts of the compositor API support a `schedule_time_ms` field to apply certain actions at a specific time. If enabled, the event will still be executed, even if it was scheduled too late. Otherwise, it will be discarded.

Defaults to `false`. Valid values: `true`, `false`, `1`, `0`.

### `LIVE_COMPOSITOR_REQUIRED_WGPU_FEATURES`

Comma separated list of WebGPU features that need to be enabled. See [https://docs.rs/wgpu/22.1.0/wgpu/struct.Features.html](https://docs.rs/wgpu/22.1.0/wgpu/struct.Features.html) for a list of available options.

Defaults to `UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING,SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING`.

Additionally, `TEXTURE_BINDING_ARRAY` and `PUSH_CONSTANTS` are also required, but this requirement can not be overwritten by changing this
environment variable.

### `LIVE_COMPOSITOR_INPUT_BUFFER_DURATION_MS`

Duration of an input buffer in milliseconds. New stream will not be processed until this buffer is filled, so this value controls the trade-off between
latency and resilience to stream delays.

This value can be safely set to 0 if either:
- All input streams are `required`
- All input streams are started with a specific `offset_ms` and you are delivering them early enough for decoding to finish.

:::warning
Increasing this value always increases the latency of the stream by the same amount.
:::

Defaults to `80ms` (about 5 frames in 60 fps).

### `LIVE_COMPOSITOR_LOG_FILE`

Path to the file were Live Compositor logs should be written. Setting this option does not disable logging to the standard output.
