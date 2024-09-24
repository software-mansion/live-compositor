import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';
import QuickStartEmpty from "./assets/quick_start_1.webp"
import QuickStartBothInputs from "./assets/quick_start_2.webp"

# Quick start

This guide will explain basic LiveCompositor setup.

## Configure inputs and output

### Start the compositor

<Tabs queryString="lang">
  <TabItem value="react" label="React">
    ```tsx
    import LiveCompositor from "@live-compositor/node"

    async function start() {
      const compositor = new LiveCompositor();
      await compositor.init();
    }
    ```
  </TabItem>
  <TabItem value="http" label="HTTP">
    Start the compositor server. Check out [configuration page](../deployment/configuration.md) for available configuration options.
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    Following code snippets are implementing `handle_init/2` or `handle_setup/2` callbacks. Those
    are just examples, you can use any [`Membrane.Pipeline` callbacks](https://hexdocs.pm/membrane_core/Membrane.Pipeline.html#callbacks)
    instead.

    ```elixir
    alias Membrane.LiveCompositor

    def handle_init(ctx, opts) do
      spec = [
        ...

        child(:live_compositor, %LiveCompositor{
          framerate: {30, 1}
        }),

        ...
      ]
      {[spec: spec], %{}}
    end
    ```
  </TabItem>
</Tabs>


### Register input stream `input_1`.

<Tabs queryString="lang">
  <TabItem value="react" label="React">
    ```tsx
    await compositor.registerInput("input_1", {
      type: "rtp_stream",
      transportProtocol: "tcp_server",
      port: 9001,
      video: {
        decoder: "ffmpeg_h264"
      },
      audio: {
        decoder: "opus"
      }
    })
    ```
    After `registerInput` call is done you can establish the connection and start sending the stream. Check out [how to deliver input streams](./deliver-input.md) to learn more.
  </TabItem>
  <TabItem value="http" label="HTTP">
    ```http
    POST: /api/input/input_1/register
    Content-Type: application/json

    {
      "type": "rtp_stream",
      "transport_protocol": "tcp_server",
      "port": 9001,
      "video": {
        "decoder": "ffmpeg_h264"
      }
      "audio": {
        "decoder": "opus"
      }
    }
    ```

    After receiving the response you can establish the connection and start sending the stream. Check out [how to deliver input streams](./deliver-input.md) to learn more.
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    ```elixir
    def handle_init(ctx, opts) do
      spec = [
        ...

        video_input_1_spec
        |> via_in(Pad.ref(:video_input, "video_input_1"))
        |> get_child(:live_compositor),

        audio_input_1_spec
        |> via_in(Pad.ref(:audio_input, "audio_input_1"))
        |> get_child(:live_compositor),

        ...
      ]
      {[spec: spec], %{}}
    end
    ```
    where `video_input_1_spec` and `audio_input_1_spec` are elements producing H264 video and Opus audio respectively.
  </TabItem>
</Tabs>

### Register input stream `input_2`.

<Tabs queryString="lang">
  <TabItem value="react" label="React">
    ```tsx
    await compositor.registerInput("input_2", {
      type: "rtp_stream",
      transportProtocol: "tcp_server",
      port: 9002,
      video: {
        decoder: "ffmpeg_h264"
      },
      audio: {
        decoder: "opus"
      }
    })
    ```
    After `registerInput` call is done you can establish the connection and start sending the stream. Check out [how to deliver input streams](./deliver-input.md) to learn more.
  </TabItem>
  <TabItem value="http" label="HTTP">
    ```http
    POST: /api/input/input_2/register
    Content-Type: application/json

    {
      "type": "rtp_stream",
      "transport_protocol": "tcp_server",
      "port": 9002,
      "video": {
        "decoder": "ffmpeg_h264"
      },
      "audio": {
        "decoder": "opus"
      }
    }
    ```

    After receiving the response you can establish the connection and start sending the stream. Check out [how to deliver input streams](./deliver-input.md) to learn more.
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    ```elixir
    def handle_init(ctx, opts) do
      spec = [
        ...

        video_input_2_spec
        |> via_in(Pad.ref(:video_input, "video_input_2"))
        |> get_child(:live_compositor),

        audio_input_2_spec
        |> via_in(Pad.ref(:audio_input, "audio_input_2"))
        |> get_child(:live_compositor),

        ...
      ]
      {[spec: spec], %{}}
    end
    ```
    where `video_input_2_spec` and `audio_input_2_spec` are elements that produce H264 video and Opus audio respectively.
  </TabItem>
</Tabs>

### Register output stream `output_1`.

Configure it to:
- render an empty [`View`](../api/components/View.md) component with a background color set to `#4d4d4d` (gray)
- produce silent audio

<Tabs queryString="lang">
  <TabItem value="react" label="React">
    ```tsx
    function App() {
      return <View backgroundColor="#4d4d4d"/>
    }

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
    After `registerOutput` is done you can establish the connection and start listening for the stream. Check out [how to receive output streams](./receive-output.md) to learn more.

    `View` component does not have any children, so on the output you should see just a blank screen
    of a specified color as shown below. There are no `InputStream` components in the scene and 
    `useAudioInput` hook was not used, so output audio will be silent.
  </TabItem>
  <TabItem value="http" label="HTTP">
    ```http
    POST: /api/output/output_1/register
    Content-Type: application/json

    {
      "type": "rtp_stream",
      "transport_protocol": "tcp_server",
      "port": 9003,
      "video": {
        "resolution": { "width": 1280, "height": 720 },
        "encoder": {
          "type": "ffmpeg_h264",
          "preset": "ultrafast"
        },
        "initial": {
          "root": {
            "type": "view",
            "background_color_rgba": "#4d4d4dff"
          }
        }
      },
      "audio": {
        "encoder": {
          "type": "opus",
          "channels": "stereo"
        },
        "initial": {
          "inputs": []
        }
      }
    }
    ```
    After receiving the response you can establish the connection and start listening for the stream. Check out [how to receive output streams](./receive-output.md) to learn more.

    `View` component does not have any children, so on the output you should see just a blank screen
    of a specified color as shown below. The `initial.inputs` list in audio config is empty, so the
    output audio will be silent.
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    ```elixir
    def handle_init(ctx, opts) do
      spec = [
        ...

        get_child(:live_compositor),
        |> via_out(Pad.ref(:video_output, "video_output_1"), options: [
          width: 1280,
          height: 720,
          encoder: %LiveCompositor.Encoder.FFmpegH264{
            preset: :ultrafast
          },
          initial: %{
            root: %{
              type: :view,
              background_color_rgba: "#4d4d4dff",
            },
          }
        ])
        |> video_output_1_spec,
        
        get_child(:live_compositor)
        |> via_out(Pad.ref(:audio_output, "audio_output_1"), options: [
          encoder: LiveCompositor.Encoder.Opus.t(),
          initial: %{
            inputs: []
          }
        ])
        |> audio_output_1_spec

        ...
      ]
      {[spec: spec], %{}}
    end
    ```
    where `video_output_1_spec` and `audio_output_1_spec` are elements that can consume H264 video and Opus audio respectively.

    You can configure output framerate and sample rate using [`framerate` and `output_sample_rate` bin options](https://hexdocs.pm/membrane_live_compositor_plugin/Membrane.LiveCompositor.html#module-bin-options).
    
    `View` component does not have any children, so on the output you should see just a blank screen
    of a specified color as shown below. The `initial.inputs` list in audio config is empty, so the
    output audio will be silent.
  </TabItem>
</Tabs>

<div style={{textAlign: 'center'}}>
    <img src={QuickStartEmpty} style={{ width: 600 }} />
    Output stream
</div>


## Update output

Configure it to:
- Show input streams `input_1` and `input_2` using [`Tiles`](../typescript/components/Tiles.md) component.
- Mix audio from input streams `input_1` and `input_2`, where `input_1` volume is slightly lowered.  

<Tabs queryString="lang">
  <TabItem value="react" label="React">
    ```tsx
    function App() {
      return (
        <Tiles backgroundColor="#4d4d4d">
          <InputStream inputId="input_1" volume={0.9} />
          <InputStream inputId="input_2" />
        </Tiles>
      )
    }
    ```
  </TabItem>
  <TabItem value="http" label="HTTP">
    ```http
    POST: /api/output/output_1/update
    Content-Type: application/json

    {
      "video": {
        "root": {
          "type": "tiles",
          "background_color_rgba": "#4d4d4dff",
          "children": [
            { "type": "input_stream", "input_id": "input_1" },
            { "type": "input_stream", "input_id": "input_2" }
          ]
        }
      },
      "audio": {
        "inputs": [
          { "input_id": "input_1", volume: 0.9 },
          { "input_id": "input_2" }
        ]
      }
    }
    ```
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    ```elixir
    def handle_setup(ctx, state) do
      video_request = %LiveCompositor.Request.UpdateVideoOutput{
        output_id: "video_output_1",
        root: %{
          type: :tiles,
          children: [
            %{ type: :input_stream, input_id: :input_1 },
            %{ type: :input_stream, input_id: :input_2 }
          ]
        }
      }
      audio_request = %LiveCompositor.Request.UpdateAudioOutput{
        output_id: "audio_output_1",
        inputs: [
          %{ input_id: "input_1", volume: 0.9 },
          %{ input_id: "input_2" }
        ]
      }
      
      events = [
        notify_child: {:live_compositor, video_request},
        notify_child: {:live_compositor, audio_request}
      ]

      {events, state}
    end
    ```
  </TabItem>
</Tabs>

<div style={{textAlign: 'center'}}>
    <img src={QuickStartBothInputs} style={{ width: 600 }} />
    Output stream
</div>
