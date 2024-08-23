import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

import QuickStartPassThrough from "./assets/basic_layouts_3.webp"
import QuickStartTiles from "./assets/basic_layouts_4.webp"

# Offline Processing

This guide will explain how to use LiveCompositor for non-real-time use cases.

By `offline processing` we mean processing in which output produced by LiveCompositor is not synchronized with a real-time clock. 

## Differences overview

There are a few differences in LiveCompositor configuration for `offline` and `live` processing.

<Tabs queryString="lang">
  <TabItem value="http" label="HTTP">

  1. To enable offline processing, the [`LIVE_COMPOSITOR_OFFLINE_PROCESSING_ENABLE`](../deployment/configuration.md#live_compositor_offline_processing_enable) environment variable should be set to `true`.
  2. [`start request`](../api/routes.md#start-request) should be sent **after** registering all inputs/outputs and sending all scheduled update/unregister requests.
  3. To avoid missing frames on inputs, register them with the `required` parameter set to `true` and `offset_ms` set to the appropriate value (if you want to start rendering input at the beginning of the output use `0`). 
  It's recommended to use MP4 as input/output protocol for non-real-time use cases. 
  If you want to use RTP, TCP is a preferred transport protocol, as it implements flow-control mechanisms.
  4. You can use a better `video.encoder.preset` option in [output register request](../api/routes.md#register-output). This option has the highest impact on output quality and performance. In offline processing LiveCompositor doesn't drop output frames when processing is slower than real-time, so you can safely choose slower presets.
  5. `schedule_time_ms` should be used in [update output requests](../api/routes.md#update-output) to achieve frame-perfect precision.
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">

  1. To enable offline processing set `composing_strategy` to `:offline_processing` when spawning the LiveCompositor bin.
  2. To avoid missing frames on inputs, link input pads with the `required` option set to `true` and `offset_ms` set to the appropriate value (if you want to start rendering input at the beginning of the output use `0`).
  3. You can use better `video.encoder.preset` option in output pads options. This option has the highest impact on output quality and performance. 
  In offline processing LiveCompositor doesn't drop output frames when processing is slower than real-time, so you can safely choose slower presets.
  4. `schedule_time` should be used in [`UpdateVideoOutput`](https://hexdocs.pm/membrane_live_compositor_plugin/Membrane.LiveCompositor.Request.UpdateVideoOutput.html) and [`UpdateAudioOutput`](https://hexdocs.pm/membrane_live_compositor_plugin/Membrane.LiveCompositor.Request.UpdateAudioOutput.html).
  </TabItem>
</Tabs>


:::warning
The `required` option will block LiveCompositor processing if input stops delivering data.
Don't use this option for unreliable inputs, as they might block LiveCompositor processing entirely.

It's not recommended to use this option in real-time processing scenarios, as it might cause latency spikes when `required` input stops delivering data.
:::


## Mixing MP4s example

### Start the compositor

<Tabs queryString="lang">
  <TabItem value="http" label="HTTP">
    Start the compositor server with the `LIVE_COMPOSITOR_OFFLINE_PROCESSING_ENABLE` environment variable set to `true`.

    Check out the [configuration page](../deployment/configuration.md) for more available configuration options.
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    Spawn LiveCompositor element with `composing_strategy` option set to `:offline_processing`.

    ```elixir
    alias Membrane.LiveCompositor

    def handle_init(ctx, opts) do
      spec = [
        ...

        child(:live_compositor, %LiveCompositor{
          framerate: {30, 1},
          // highlight-start
          composing_strategy: :offline_processing
          // highlight-end
        }),

        ...
      ]
      {[spec: spec], %{}}
    end
    ```
  </TabItem>
</Tabs>

### Add first input `input_1`

<Tabs queryString="lang">
  <TabItem value="http" label="HTTP">
    ```http
    POST: /api/input/input_1/register
    Content-Type: application/json

    {
      "type": "mp4",
      "path": "input_1.mp4",
      // highlight-start
      "required": true,
      "offset_ms": 0
      // highlight-end
    }
    ```
    Notice that:
    - The `required` field is set to `true` to prevent dropping any input frames from `input_1` input.
    - The `offset_ms` field is set to `0`, as we want to start displaying this input from the first output frame
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    ```elixir
    def handle_init(ctx, opts) do
      spec = [
        ...

        video_input_1_spec
        |> via_in(Pad.ref(:video_input, "video_input_1"),
          // highlight-start
          options: [
            offset: Membrane.Time.seconds(0),
            required: true
          ]
          // highlight-end
        )
        |> get_child(:live_compositor),

        audio_input_1_spec
        |> via_in(Pad.ref(:audio_input, "audio_input_1"),
          // highlight-start
          options: [
            offset: Membrane.Time.seconds(0),
            required: true
          ]
          // highlight-end
        )
        |> get_child(:live_compositor),

        ...
      ]
      {[spec: spec], %{}}
    end
    ```
    where `video_input_1_spec` and `audio_input_1_spec` are elements producing H264 video and Opus audio respectively.

    Notice that:
    - the `required` option is set to `true` to prevent dropping any input frames from the `input_1` input
    - `offset` is set to `0s`, as we want to start displaying this input from the first output frame

    If you want to use MP4 as an input and output, check out [this example](https://github.com/membraneframework/membrane_live_compositor_plugin/blob/master/examples/lib/offline_processing.exs) for complete pipeline setup.
  </TabItem>
</Tabs>

### Add second input `input_2` with offset

<Tabs queryString="lang">
  <TabItem value="http" label="HTTP">
    ```http
    POST: /api/input/input_2/register
    Content-Type: application/json

    {
      "type": "mp4",
      "url": "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/input_2.mp4"
      "required": true,
      // highlight-start
      "offset_ms": 5000
      // highlight-end
    }
    ```
    Here we are going to set `offset_ms` to `5000` to start playing this input 5s after the start of the output.
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    ```elixir
    def handle_init(ctx, opts) do
      spec = [
        ...

        video_input_1_spec
        |> via_in(Pad.ref(:video_input, "video_input_2"),
          options: [
            // highlight-start
            offset: Membrane.Time.seconds(5),
            // highlight-end
            required: true
          ]
        )
        |> get_child(:live_compositor),

        audio_input_1_spec
        |> via_in(Pad.ref(:audio_input, "audio_input_2"),
          options: [
            // highlight-start
            offset: Membrane.Time.seconds(5),
            // highlight-end
            required: true
          ]
        )
        |> get_child(:live_compositor),

        ...
      ]
      {[spec: spec], %{}}
    end
    ```
    where `video_input_1_spec` and `audio_input_1_spec` are elements producing H264 video and Opus audio respectively.

    Here we are going to set `offset` to `5s` to start playing this input 5s after the start.
  </TabItem>
</Tabs>

### Add MP4 output `output_1`

Here we won't focus on configuring the composition itself, as it's the same as in any other LiveCompositor use case.
For more advanced layouts, check out the [`Basic Layouts guide`](./basic-layouts.md).

We will configure the output to simply:
- Rescale video from `input_1` to output resolution using the [`Rescaler`](../api/components/Rescaler.md) component.
- Passthrough audio from `input_1` input.

<Tabs queryString="lang">
  <TabItem value="http" label="HTTP">
    ```http
    POST: /api/output/output_1/register
    Content-Type: application/json

    {
      "type": "mp4",
      "path": "output.mp4",
      "video": {
        "encoder": {
          "type": "ffmpeg_h264",
          "preset": "medium"
        },
        "resolution": {
          "width": 1280,
          "height": 720
        },
        "initial": {
          "root": {
            "type": "rescaler",
            "children": [{
              "type": "input_stream",
              "input_id": "input_1"
            }]
          }
        }
      },
      "audio": {
        "encoder": {
          "type": "aac",
          "channels": "stereo"
        },
        "initial": {
          "inputs": [
            { "input_id": "input_1" }
          ]
        }
      }
    }
    ```
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
            root: {
              type: :rescaler,
              children: [{
                type: :input_stream,
                input_id: "input_1"
              }]
            },
          }
        ])
        |> video_output_1_spec,
        
        get_child(:live_compositor)
        |> via_out(Pad.ref(:audio_output, "audio_output_1"), options: [
          encoder: LiveCompositor.Encoder.Opus.t(),
          initial: %{
            inputs: [
              %{input_id: "input_1"}
            ]
          }
        ])
        |> audio_output_1_spec

        ...
      ]
      {[spec: spec], %{}}
    end
    ```
  </TabItem>
</Tabs>

### Schedule update output to show both inputs

We are going to schedule output update 5s after the begging and configure it to:
- Show both inputs side-by-side using the [`Tiles`](../api/components/Tiles.md) component.
- Mix audio from both inputs, where `input_2` volume is reduced to 35% of the original volume.

<Tabs queryString="lang">
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
          { "input_id": "input_1" },
          { "input_id": "input_2", "volume": 0.35 }
        ]
      },
      // highlight-start
      "schedule_time_ms": 5000
      // highlight-end
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
            %{ type: :input_stream, input_id: "input_1" },
            %{ type: :input_stream, input_id: "input_2" }
          ]
        },
        // highlight-start
        schedule_time: Membrane.Time.seconds(5)
        // highlight-end
      }
      audio_request = %LiveCompositor.Request.UpdateAudioOutput{
        output_id: "audio_output_1",
        inputs: [
          %{ input_id: "input_1" },
          %{ input_id: "input_2", volume: 0.35 }
        ],
        // highlight-start
        schedule_time: Membrane.Time.seconds(5)
        // highlight-end
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

### Scheduling the end of the processing

<Tabs queryString="lang">
  <TabItem value="http" label="HTTP">
    There are 2 mechanisms to define when input should end:

    1. Sending/scheduling an [unregister output request](../api/routes.md#unregister-output). You should use this option if know the desired duration of the output.
    
    To schedule output unregister after 10s you simply send:
    ```http
    POST: /api/output/output_1/unregister
    Content-Type: application/json

    {
      // highlight-start
      "schedule_time_ms": 10_000
      // highlight-end
    }
    ```

    2. Using `send_eos_when` option in [`register_output.video`](../api/generated/output-Mp4Output.md#outputvideooptions) and [`register_output.audio`](../api/generated/output-Mp4Output.md#outputmp4audiooptions). You should use this option to end output after any/all/some inputs end.

    To end output after all inputs end, you need to modify the register output request we sent previously:
    ```http
    POST: /api/output/output_1/register
    Content-Type: application/json

    {
      "type": "mp4",
      "path": "output.mp4",
      "video": {
        ...,
        // highlight-start
        send_eos_when: {
          all_inputs: true
        }
        // highlight-end
      },
      "audio": {
        ...,
        // highlight-start
        send_eos_when: {
          all_inputs: true
        }
        // highlight-end
      }
    }
    ```

  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    There are 2 mechanisms to define when input should end:

    1. Sending/scheduling an [unregister output request](https://hexdocs.pm/membrane_live_compositor_plugin/Membrane.LiveCompositor.Request.UnregisterOutput.html). You should use this option if you know the desired duration of the output.

    ```elixir
    def handle_setup(ctx, state) do
      scheduled_unregister_video = %LiveCompositor.UnregisterOutput{
        output_id: "video_output_1",
        schedule_time: Membrane.Time.seconds(5)
      }
      scheduled_unregister_audio = %LiveCompositor.UnregisterOutput{
        output_id: "audio_output_1",
        schedule_time: Membrane.Time.seconds(5)
      }
      actions = [
        notify_child: {:live_compositor, scheduled_unregister_video},
        notify_child: {:live_compositor, scheduled_unregister_audio},
      ]

      {actions, state}
    end
    ```
    
    To schedule output unregister after 10s you simply send:
    ```elixir
    POST: /api/output/output_1/unregister
    Content-Type: application/json

    {
      // highlight-start
      "schedule_time_ms": 10_000
      // highlight-end
    }
    ```

    2. Using `send_eos_when` option on `:video_output` and `:audio_output` options. You should use this option to end output after any/all/some inputs end.

    To end output after all inputs end, you need to modify the output spec we configured previously:
    ```elixir
    def handle_init(ctx, opts) do
      spec = [
        ...

        get_child(:live_compositor),
        |> via_out(Pad.ref(:video_output, "video_output_1"), options: [
          ...,
          // highlight-start
          send_eos_when: :all_inputs
          // highlight-end
        ])
        |> video_output_1_spec,
        
        get_child(:live_compositor)
        |> via_out(Pad.ref(:audio_output, "audio_output_1"), options: [
          ...,
          // highlight-start
          send_eos_when: :all_inputs
          // highlight-end
        ])
        |> audio_output_1_spec

        ...
      ]
      {[spec: spec], %{}}
    end
    ```
  </TabItem>
</Tabs>

### Waiting for processing to finish

<Tabs queryString="lang">
  <TabItem value="http" label="HTTP">
    To detect when LiveCompositor finishes processing and it's safe to shut it down, you should subscribe to a [WebSocket endpoint](../api/routes.md#websocket-endpoint) and wait for the [OutputDone event](../api/events.md#output_done).
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    LiveCompositor bin will send [`Membrane.Action.end_of_stream()`](https://hexdocs.pm/membrane_core/Membrane.Element.Action.html#t:end_of_stream/0).
    Most elements handle this automatically, so you don't need to worry about it unless you are using custom elements.
  </TabItem>
</Tabs>

### Start processing

<Tabs queryString="lang">
  <TabItem value="http" label="HTTP">
    You should send a [`start request`](../api/routes.md#start-request) **after** registering all inputs/outputs and scheduling all requests.
    If you send it before, some input frames might be dropped and some update requests might not be applied properly.

    ```http
    POST: /api/start
    Content-Type: application/json

    {}
    ```
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    LiveCompositor bin starts composing automatically after spawning, so it's important to register all inputs/outputs and register all updates in [`handle_init`](https://hexdocs.pm/membrane_core/Membrane.Pipeline.html#c:handle_init/2) or [`handle_setup`](https://hexdocs.pm/membrane_core/Membrane.Pipeline.html#c:handle_setup/2) callbacks - before spawned bin will start processing buffers.
  </TabItem>
</Tabs>



