import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';
import QuickStartEmpty from "../assets/quick_start_1.webp"
import QuickStartBothInputs from "../assets/quick_start_2.webp"

# Quick start

This guide will explain basics of the LiveCompositor and help you get started quickly.

## Configure inputs and output

### Start the compositor

Let's start by creating a new `LiveCompositor` instance and initializing it.

```tsx
import LiveCompositor from "@live-compositor/node"

async function start() {
  const compositor = new LiveCompositor();
  await compositor.init();
}
```

By default, it will download binaries and start the LiveCompositor server on the local machine. Alternatively,
you can connect to an existing instance by providing appropriate connection manager:

<details>
    <summary>Spawning server locally (default behaviour)</summary>

    ```tsx
    import LiveCompositor, { LocallySpawnedInstance } from "@live-compositor/node"

    async function start() {
      const connectionManager = new LocallySpawnedInstance({
        port: 8000,
      });
      const compositor = new LiveCompositor(connectionManager);
      await compositor.init();
    }
    ```
</details>

<details>
    <summary>Connect to existing instance</summary>

    ```tsx
    import LiveCompositor, { ExistingInstance } from "@live-compositor/node"

    async function start() {
      const connectionManager = new ExistingInstance({
        ip: '127.0.0.1',
        port: 8000,
        protocol: 'http'
      });
      const compositor = new LiveCompositor(connectionManager);
      await compositor.init();
    }
    ```
</details>

### Register input stream `input_1` and `input_2`.

Now we need to add some media that can be later composed to produce output stream. In this guide we will use 2 inputs (`input_1` and `input_2`).

```tsx
async function start() {
  // Compositor init code ...

  await compositor.registerInput("input_1", {
    type: "rtp_stream",
    transportProtocol: "tcp_server",
    port: 9001,
    video: { decoder: "ffmpeg_h264" },
    audio: { decoder: "opus" }
  })

  await compositor.registerInput("input_2", {
    type: "rtp_stream",
    transportProtocol: "tcp_server",
    port: 9002,
    video: { decoder: "ffmpeg_h264" },
    audio: { decoder: "opus" }
  })
}
```

After that you can establish the connection and start sending the stream. Check out [how to deliver input streams](../deliver-input.md) to learn more.

Above is an example of using RTP over TCP, but can use MP4 input instead:

<details>
    <summary>Alternative: MP4 inputs</summary>

    ```tsx
    async function start() {
      // Compositor init code ...

      await compositor.registerInput("input_1", {
        type: "mp4",
        serverPath: "example.mp4"
      })

      await compositor.registerInput("input_2", {
        type: "mp4",
        serverPath: "example.mp4"
      })
    }
    ```
</details>

### Register output stream `output_1`.

At this point we have everything ready to start streaming.

Define the `App` component that will represent our scene. For now, it returns
just a simple [`View`](../../api/components/View.md) with a background.

```tsx
import { View } from "live-compositor"

function App() {
  return <View backgroundColor="#4d4d4d" />
}
```

Register new output with [`registerOutput`](../../typescript/instance.md#register-output) that uses the `App` component:

```tsx

async function start() {
  // init code from previous steps

  await compositor.registerOutput("output_1", {
    type: "rtp_stream",
    transportProtocol: "tcp_server",
    port: 9003,
    video: {
      resolution: { width: 1280, height: 720 },
      encoder": {
        type: "ffmpeg_h264",
        preset: "ultrafast"
      },
      root: <App />
    },
    audio: {
      encoder: {
        type: "opus",
        channels: "stereo"
      },
    }
  })
}
```


After that you can establish the connection and start listening for the stream. Check out [how to receive output streams](../receive-output.md) to learn more.

On the output you should see just a blank screen of a specified color as shown below. There are no `InputStream` components in the scene and
`useAudioInput` hook was not used, so output audio will be silent.

<div style={{textAlign: 'center'}}>
    <img src={QuickStartEmpty} style={{ width: 600 }} />
    Output stream
</div>


## Use inputs when composing 

In the previous steps we produced the output stream, but none of the inputs were shown. Let's add them 
by modifying `App` component from previous step.

```tsx
import { Tiles, InputStream } from "live-compositor"

function App() {
  return (
    <Tiles backgroundColor="#4d4d4d">
      <InputStream inputId="input_1" volume={0.9} />
      <InputStream inputId="input_2" />
    </Tiles>
  )
}
```

On the output stream we can see both input streams next to each other and hear audio from both of them where `input_1` volume is slightly lowered.

<div style={{textAlign: 'center'}}>
    <img src={QuickStartBothInputs} style={{ width: 600 }} />
    Output stream
</div>
