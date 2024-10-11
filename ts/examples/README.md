# `live-compositor` examples

## Node.js examples

- `./node-examples/src/simple.tsx` - Basic example with `Text`/`View` components and basic state usage.
- `./node-examples/src/news-ticker.tsx` - Example of infinite scrolling text line.
- `./node-examples/src/dynamic-text.tsx` - Example of text that dynamically grows and shrinks in a loop.
- `./node-examples/src/dynamic-inputs.tsx` - Example of using `useInputStreams` hook to handle new inputs.

### Usage

To launch any of the above examples go to `node-examples` directory and run:

```bash
pnpm run ts-node <path-to-example>
```

e.g.

```bash
pnpm run ts-node ./src/simple.tsx
```
