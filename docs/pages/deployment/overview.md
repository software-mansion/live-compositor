# Deployment

LiveCompositor can be deployed in various ways, depending on your platform, used features and whether it is used standalone or via Membrane Framework plugin.

You can consider following options for LiveCompositor deployment:
- Using docker
  - (recommended) Dockerfile with compositor without web rendering support [https://github.com/software-mansion/live-compositor/blob/master/build_tools/docker/slim.Dockerfile](https://github.com/software-mansion/live-compositor/blob/master/build_tools/docker/slim.Dockerfile)
  - Dockerfile with compositor with web rendering support [https://github.com/software-mansion/live-compositor/blob/master/build_tools/docker/full.Dockerfile](https://github.com/software-mansion/live-compositor/blob/master/build_tools/docker/full.Dockerfile)
- Standalone binaries
  - Building [`github.com/software-mansion/live-compositor`](https://github.com/software-mansion/live-compositor) from source.
  - Binaries from [GitHub releases](https://github.com/software-mansion/live-compositor/releases).
- As an element in a Membrane pipeline. [Learn more.](#membrane-framework-plugin)

## Requirements

See [`requirements`](./requirements.md) page for details about software and hardware requirements of the compositor in various configurations.

## Configuration

Some of the compositor behaviors can only be configured on server startup. You can define those options using `LIVE_COMPOSITOR_*`
environment variables. Full list of those variables can be found [here](./configuration.md).

## Web renderer support

If you want to use a [`WebView`](../api/components/WebView.md) component in your scene definition you need to use binaries compiled
with web rendering support.
- When building from source you need to have `web_renderer` feature enabled (enabled by default).
- When using binaries from [GitHub releases](https://github.com/software-mansion/live-compositor/releases) use files with `_with_web_renderer_` in the name.
- When using Docker use [github.com/software-mansion/live-compositor/blob/master/build_tools/docker/full.Dockerfile](https://github.com/software-mansion/live-compositor/blob/master/build_tools/docker/full.Dockerfile)

:::warning
Keep in mind that using a browser for rendering might not be secure, especially if you use it to render untrusted websites
or content provided by the end user. Unless you specifically need that capability, we recommend using LiveCompositor without
web rendering support.
:::

## DeckLink support

If you want to use a DeckLink device as an input you need to use binaries compiled with support for it. When building from
source you need to have `decklink` feature enabled (enabled by default).

Currently, we do not provide binaries or Dockerfiles with DeckLink support, and only support x86_64 Linux platform.

## Membrane Framework plugin

#### Requirements

By default, Membrane plugin is using a binary release, so in most cases [this section](./requirements.md#binaries-from-github-releases)
applies, but you can also override LiveCompositor binary used by the plugin and provide your own.

#### Configuration

Membrane Framework plugin provides a way to define a compositor configuration. However, there are configuration options
that are not exposed with the plugin API. In most cases it should not be needed, but if you need to set option like that you can always
configure compositor with environment variables.

#### Web renderer support

Default binary used by the plugin was built without web rendering support. To use web rendering inside the plugin you need to override
the compositor binary.

#### DeckLink support

Not supported
