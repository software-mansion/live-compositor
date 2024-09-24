# `LiveCompositor`

Packages like `@live-compositor/node` export `LiveCompositor` class that is a main entity used to interact with or control Live Compositor server instance.

```tsx
import LiveCompositor from "@live-compositor/node"
```

### `new LiveCompositor()`

```tsx
new LiveCompositor(manager?: CompositorManager)
```

Creates new compositor configuration. You have to call `init` first before this object can be used.


`CompositorManager` configures how the client code will connect and manage the LiveCompositor server.
- (**default**) `LocallySpawnedInstance` from `@live-compositor/node` downloads LiveCompositor binaries and starts the server on the local machine.
- `ExistingInstance` from `@live-compositor/node` connects to already existing compositor instance.

### `init()`

```tsx
LiveCompositor.init(): Promise<void>
```

Initialize the LiveCompositor instance, depending on which `CompositorManager` you are using it might mean spawning
new instance or just establishing connection.

After this request you can start connecting inputs/outputs or register other elements. However, no output stream will
be produced until `start()` method is called.

### `start()`

```tsx
LiveCompositor.start(): Promise<void>
```

Starts the processing pipeline. Any previously registered output will start producing the video/audio stream.

***

### Outputs configuration

#### Register output

```tsx
import { Outputs } from "live-compositor"

LiveCompositor.registerOutput(
  outputId: string,
  output: Outputs.RegisterOutput,
): Promise<object>
```

Register external destination that can be used as a compositor output. See outputs documentation to learn more:
- [Rtp](./outputs/rtp.md)
- [Mp4](./outputs/mp4.md)

#### Unregister output

```tsx
LiveCompositor.unregisterOutput(outputId: string): Promise<void>
```

Unregister previously registered output.

***

### Inputs configuration

#### Register input

```tsx
import { Inputs } from "live-compositor"

LiveCompositor.registerInput(
  inputId: string,
  input: Inputs.RegisterInput,
): Promise<object>
```

Register external source that can be used as a compositor input. See inputs documentation to learn more:
- [RTP](./inputs/rtp.md)
- [MP4](./inputs/mp4.md)

#### Unregister input

```tsx
LiveCompositor.unregisterInput(inputId: string): Promise<void>
```

Unregister a previously registered input.

***

### Renderers configuration

#### Register image

```tsx
import { Renderers } from "live-compositor"

LiveCompositor.registerImage(
  imageId: string,
  image: Renderers.RegisterImage,
): Promise<void>
```

Register an image asset. See [`Renderers.RegisterImage`](./renderers/image.md) to learn more.

#### Unregister image

```tsx
LiveCompositor.unregisterImage(imageId: string): Promise<void>
```

Unregister a previously registered image asset.

#### Register shader

```tsx
import { Renderers } from "live-compositor"

LiveCompositor.registerShader(
  shaderId: string,
  shader: Renderers.RegisterShader,
): Promise<void>
```

Register a shader. See [`Renderers.RegisterShader`](./renderers/shader.md) to learn more.

#### Unregister shader

```tsx
LiveCompositor.unregisterShader(shaderId: string): Promise<void>
```

Unregister a previously registered shader.

#### Register web renderer instance

```tsx
import { Renderers } from "live-compositor"

LiveCompositor.registerWebRenderer(
  instanceId: string,
  instance: Renderers.RegisterWebRenderer,
): Promise<object>
```

Register a web renderer instance. See [`Renderer.RegisterWebRenderer`](./renderers/web.md) to learn more.

#### Unregister web renderer instance

```tsx
LiveCompositor.unregisterWebRenderer(instanceId: string): Promise<void>
```

Unregister a previously registered web renderer.
