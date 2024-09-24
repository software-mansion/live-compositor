# TypeScript SDK

## Packages

- `live-compositor` - Provide core React components that can be used to define a composition of a video stream.
- `@live-compositor/node` - Provides API to create and manage LiveCompositor instances for Node.js environment.
- `@live-compositor/core` - Shared runtime-independent implementation that is used by packages like `@live-compositor/node`.
- `@live-compositor/browser-render` - Rendering implementation from LiveCompositor compiled to WASM and uses WebGL as rendering backend.

## Examples

- `./examples/node-examples` - Few examples of using LiveCompositor with TypeScript from Node.js
- `./examples/vite-browser-render` - Example of using `@live-compositor/browser-render` package in a Vite project.
