---
description: React Hooks available provided by LiveCompositor SDK
---

# Hooks

When you define how the LiveCompositor should compose streams you can use regular React hooks like `useState`
or `useEffect`, but this SDK provides few compositor specific hooks to interact with the audio/video.

## `useInputStreams`

```tsx
type InputStreamInfo = {
  inputId: InputId;
  videoState?: 'ready' | 'playing' | 'finished';
  audioState?: 'ready' | 'playing' | 'finished';
}

function useInputStreams(): Record<InputId, InputStreamInfo>: 
```

`useInputStreams` returns an object representing connected streams and their current state.

## `useInputAudio`

```tsx
type AudioOptions = {
  volume: number
}

function useAudioInput(inputId: Api.InputId, audioOptions: AudioOptions);
```

Hook used to control audio configuration. If you already placing [`InputStream`](./components/InputStream.md) component
you can use `mute` and `volume` props instead.

Adding this hook more than once for the specific input will sum the volume.

- `AudioOptions.volume` - number between 0 and 1 representing the audio volume.
