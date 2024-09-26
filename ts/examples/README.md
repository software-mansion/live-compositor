# `live-compositor` examples

## Node.js examples

- `./node-examples/src/simple.tsx` - Basic example with `Text`/`View` components and basic state usage.
- `./node-examples/src/dynamic-text.tsx` - Example of text that dynamically grows and shrinks in a loop.
- `./node-examples/src/dynamic-inputs.tsx` - Example of using `useInputStreams` hook to handle new inputs.
- `./node-examples/src/dynamic-outputs.tsx` - Example of adding new output stream after LiveCompositor was started.
- `./node-examples/src/audio.tsx` - Example of mixing audio. (This example is using GStreamer to receive the output stream).

### Usage

To launch any of the above examples go to `node-examples` directory and run:

```bash
npm run ts-node <path-to-example>
```

e.g.

```bash
npm run ts-node ./src/simple.tsx
```
