# Changelog

## unreleased

### 💥 Breaking changes

- Drop support for `SHADER_UNUSED_VERTEX_OUTPUT` `wgpu` feature.  ([#733](https://github.com/software-mansion/live-compositor/pull/733)) by [@jerzywilczek](https://github.com/jerzywilczek)

### ✨ New features

- Add `loop` option for MP4 input. ([#699](https://github.com/software-mansion/live-compositor/pull/699) by [@WojciechBarczynski](https://github.com/WojciechBarczynski))
- Add `LIVE_COMPOSITOR_LOG_FILE` environment variable to enable logging to file ([#853](https://github.com/software-mansion/live-compositor/pull/853) by [@wkozyra95](https://github.com/wkozyra95))
- Add border, border radius and box shadow options to `Rescaler` and `View` components. ([#815](https://github.com/software-mansion/live-compositor/pull/815) by [@WojciechBarczynski](https://github.com/WojciechBarczynski), ([#839](https://github.com/software-mansion/live-compositor/pull/839), [#842](https://github.com/software-mansion/live-compositor/pull/842), [#858](https://github.com/software-mansion/live-compositor/pull/858) by [@wkozyra95](https://github.com/wkozyra95))


### 🐛 Bug fixes

- Fix AAC output unregister before the first sample. ([#714](https://github.com/software-mansion/live-compositor/pull/714) by [@WojciechBarczynski](https://github.com/WojciechBarczynskipull/714))
- Fix output mp4 timestamps when output is registered after pipeline start. ([#731](https://github.com/software-mansion/live-compositor/pull/731) by [@WojciechBarczynski](https://github.com/WojciechBarczynski))

### 🔧 Others

- Automatically rename file under the output path for MP4 output if it already exists. ([#684](https://github.com/software-mansion/live-compositor/pull/684) by [@WojciechBarczynski](https://github.com/WojciechBarczynski))
- Make `video.encoder.preset` optional in the output register. ([#782](https://github.com/software-mansion/live-compositor/pull/782) by [@WojciechBarczynski](https://github.com/WojciechBarczynski))

## [v0.3.0](https://github.com/software-mansion/live-compositor/releases/tag/v0.3.0)

### 💥 Breaking changes

- Remove `forward_error_correction` option from RTP OPUS output. ([#615](https://github.com/software-mansion/live-compositor/pull/615) by [@wkozyra95](https://github.com/wkozyra95))

### ✨ New features

- Support DeckLink cards as an input. ([#587](https://github.com/software-mansion/live-compositor/pull/587), [#597](https://github.com/software-mansion/live-compositor/pull/597), [#598](https://github.com/software-mansion/live-compositor/pull/598), [#599](https://github.com/software-mansion/live-compositor/pull/599) by [@wkozyra95](https://github.com/wkozyra95))
- Add `LIVE_COMPOSITOR_INPUT_BUFFER_DURATION_MS` environment variable to control input stream buffer size. ([#600](https://github.com/software-mansion/live-compositor/pull/600) by [@wkozyra95](https://github.com/wkozyra95))
- Add endpoint for requesting keyframe on the output stream. ([#620](https://github.com/software-mansion/live-compositor/pull/620) by [@WojciechBarczynski](https://github.com/WojciechBarczynski))
- Add MP4 output ([#657](https://github.com/software-mansion/live-compositor/pull/657) by [@WojciechBarczynski](https://github.com/WojciechBarczynski))
- Add `OUTPUT_DONE` WebSocket event ([#658](https://github.com/software-mansion/live-compositor/pull/658) by [@WojciechBarczynski](https://github.com/WojciechBarczynski))

### 🐛 Bug fixes

- Fix input queueing when some of the inputs do not produce frames/samples . ([#625](https://github.com/software-mansion/live-compositor/pull/625) by [@wkozyra95](https://github.com/wkozyra95))

## [v0.2.0](https://github.com/software-mansion/live-compositor/releases/tag/v0.2.0)

Initial release
