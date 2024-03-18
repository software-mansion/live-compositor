# Configuration

## Environment variables

### `LIVE_COMPOSITOR_API_PORT`

API port. Defaults to 8001.

### `LIVE_COMPOSITOR_INSTANCE_ID`

ID that will be returned in `GET /status` request. Can be used to identify if we are connecting to the correct compositor instance.

### `LIVE_COMPOSITOR_OUTPUT_FRAMERATE`

Output framerate for all output streams. This value can be a number or string in the `NUM/DEN` format, where both `NUM` and `DEN` are unsigned integers.

### `LIVE_COMPOSITOR_OUTPUT_SAMPLE_RATE`

Output sample rate for all output streams. This value has to be a number or string representing supported sample rate.

Supported sample rates are: 8000, 12000, 16000, 24000, 48000

### `LIVE_COMPOSITOR_FORCE_GPU`

If enabled, GPU will be required for rendering. If only CPU based adapters will be found then process will exit with an error. Defaults to `false`.

### `LIVE_COMPOSITOR_STREAM_FALLBACK_TIMEOUT_MS`

A timeout that defines when the compositor should switch to fallback on the input stream that stopped sending frames.

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
This option does not apply to logs produced by `FFmpeg` or the embedded Chromium instance used for web rendering.
:::

### `LIVE_COMPOSITOR_FFMPEG_LOGGER_LEVEL`

Minimal log level that should be logged. Supported options:
- `error` - equivalent to FFmpeg's `error, 16`
- `warn` - equivalent to FFmpeg's `warning, 24`
- `info` - equivalent to FFmpeg's `info, 32`
- `debug` - equivalent to FFmpeg's `debug, 48`
 

See `-loglevel` option in [FFmpeg documentation](https://ffmpeg.org/ffmpeg.html).

### `LIVE_COMPOSITOR_DOWNLOAD_DIR`

A path to a directory in which downloaded files will be stored. Uses the location provided by the OS if not set.

In this directory, an instance of the compositor will create a subdirectory named `live-compositor-<random number>`. Downloaded temporary files will be stored there.

### `LIVE_COMPOSITOR_WEB_RENDERER_ENABLE`

Enable web rendering capabilities. With this option disabled, you can not use [`WebView` components](../api/components/WebView) or register [`WebRenderer` instances](../api/renderers/web).

Defaults to `false`. Valid values: `true`, `false`, `1`, `0`.

### `LIVE_COMPOSITOR_WEB_RENDERER_GPU_ENABLE`

Enable GPU support inside the embedded Chromium instance.

### `LIVE_COMPOSITOR_AHEAD_OF_TIME_PROCESSING_ENABLE`

If enabled, the LiveCompositor server will try to generate output frames ahead of time if all inputs are available.

When to enable this option:
- If you want to process input streams faster than in real time.

Defaults to `false`. Valid values: `true`, `false`, `1`, `0`.

### `LIVE_COMPOSITOR_RUN_LATE_SCHEDULED_EVENTS`

Parts of the compositor API support a `schedule_time_ms` field to apply certain actions at a specific time. If enabled, the event will still be executed, even if it was scheduled too late. Otherwise, it will be discarded.

Defaults to `false`. Valid values: `true`, `false`, `1`, `0`.
