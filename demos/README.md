# LiveCompositor TypeScript demos

## Technical requirements

- **FFmpeg**
- **Gstreamer**
- NodeJS + npm

Before running demos, install JS dependencies with:

```console
npm install
```

## Demos

### 1. Video Conferencing

Run this example with:

```console
npm run 01-videoconferencing
```

This example simulates composing video conference footage.
It demonstrate how you can change output dynamically with smooth transitions.

This example also use your webcam. If you have problems with webcam footage, you can substitute it with prerecorded mp4 file:

```console
export LIVE_COMPOSITOR_WEBCAM=false
```

### 2. TV Broadcast

Run this example with:

```console
npm run 02-tv_broadcast
```

This example simulates TV broadcasting scenario.
It demonstrate how you can combine build-in components with own shaders, customizing LiveCompositor for specific use-case, while utilizing GPU rendering acceleration.
In this example, green-screen is removed from input stream with use of custom shader. Transformed input stream, background image, logo, and text are combined in output stream.

### 3. Screen stream

Run this example with:

```console
npm run 03-screen_stream
```

This example simulates live streaming screen footage with webcam.
It demonstrate how to setup simple output and add elements like donate notifications.

This example also use your webcam. If you have problems with webcam footage, you can substitute it with prerecorded mp4 file:

```console
export LIVE_COMPOSITOR_WEBCAM=false
```

## Learn more

You can learn more from [documentation](https://compositor.live/docs/intro).
API reference and guides can be found there.
