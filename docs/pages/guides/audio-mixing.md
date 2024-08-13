import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

# Audio mixing

This guide will explain how to mix audio.

LiveCompositor supports audio streams and provides a simple mixer to combine them. Even if you only have one audio stream and do not need to modify it in any way, then it is still good to pass that stream to the compositor to avoid synchronization issues between audio and video.

## Configure input and output

### Start the compositor

<Tabs queryString="lang">
  <TabItem value="http" label="HTTP">
    [Start the compositor server](../deployment/overview.md). Check out [configuration page](../deployment/configuration.md) for available configuration options.
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

### Register input `input_1` with video and audio

<Tabs queryString="lang">
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
      },
      "audio": {
        "decoder": "opus"
      }
    }
    ```

    After receiving the response you can establish the connection and start sending the stream. Check out [how to deliver input streams](./deliver-input.md) to learn more.

    In this example, we are using RTP over TCP, but it could be easily replaced by UDP.
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

    Note that `:video_input` and `:audio_input` are separate pads and should have unique names.
  </TabItem>
</Tabs>


### Register input `input_2` with only audio

<Tabs queryString="lang">
  <TabItem value="http" label="HTTP">
    ```http
    POST: /api/input/input_2/register
    Content-Type: application/json

    {
      "type": "rtp_stream",
      "transport_protocol": "tcp_server",
      "port": 9002,
      "audio": {
        "decoder": "opus"
      }
    }
    ```
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    ```elixir
    def handle_init(ctx, opts) do
      spec = [
        ...

        audio_input_2_spec
        |> via_in(Pad.ref(:audio_input, "audio_input_2"))
        |> get_child(:live_compositor),

        ...
      ]
      {[spec: spec], %{}}
    end
    ```
    where `audio_input_2_spec` is an element that produces Opus audio.

    Using Membrane, video and audio inputs should be a separate pads.
  </TabItem>
</Tabs>

### Register output `output_1`

Configure it to:
- pass through video from `input_1` with [`Text`](../api/components/Text.md) overlay  
- mix audio from `input_1` and audio from `input_2` with reduced volume to 90% of the original volume. 

<Tabs queryString="lang">
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
          "preset": "ultrafast",
        },
        "initial": {
          "root": {
            "type": "view",
            "children": [
              {
                "type": "rescaler",
                "width": 1280,
                "height": 720,
                "top": 0,
                "left": 0,
                "child": {
                  "type": "input_stream",
                  "input_id": "input_1"
                }
              },
              {
                "type": "view",
                "width": 1280,
                "height": 100,
                "background_color_rgba": "#40E0D0FF",
                "children": [{
                  "type": "text",
                  "text": "LiveCompositor 🚀😃",
                  "font_size": 80,
                  "align": "center"
                }]
              }
            ]
          }
        }
      },
      "audio": {
        "initial": {
            "inputs": [
                { "input_id": "audio_input_1" },
                { "input_id": "audio_input_2", "volume": 0.9 }
            ]
        },
        "encoder": {
          "type": "opus",
          "channels": "stereo"
        },

      }
    }
    ```
    You can configure output sample rate with the [`LIVE_COMPOSITOR_OUTPUT_SAMPLE_RATE` environmental variable](../deployment/configuration.md#live_compositor_output_sample_rate).

    After receiving the response you can establish the connection and start listening for the stream. Check out [how to receive output streams](./receive-output.md) to learn more.

    In this example we are using RTP over TCP. If you prefer to use UDP you need to start listening on the specified port before sending the register request to make sure you are not losing first frames.
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    ```elixir
    def handle_init(ctx, opts) do
      spec = [
        ...
        get_child(:live_compositor)
        |> via_out(Pad.ref(:video_output, "video_output_1|), options: [
          encoder: %LiveCompositor.Encoder.FFmpegH264{
            preset: :ultrafast
          }
        ])

        get_child(:live_compositor),
        |> via_out(Pad.ref(:audio_output, "audio_output_1"), options: [
          encoder: %LiveCompositor.Encoder.Opus{
            channels: :stereo
          },
          initial: %{
            inputs: [
              %{input_id: "audio_input_1"},
              %{input_id: "audio_input_2", volume: 0.9}
            ]
          }
        ])
        |> audio_output_1_spec

        ...
      ]
      {[spec: spec], %{}}
    end
    ```
    where `video_output_1_spec` and `audio_output_1_spec` are elements consuming H264 video and Opus audio respectively.
  </TabItem>
</Tabs>

### Update output `output_1`

Forward video and audio from `input_1` to output `output_1`.

<Tabs queryString="lang">
  <TabItem value="http" label="HTTP">
    ```http
    POST: /api/output/output_1/update
    Content-Type: application/json

    {
      "video": {
        "root": {
          "type": "rescaler",
          "child": {
            "type": "input_stream",
            "input_id": "input_1"
          }
        }
      },
      "audio": {
        "inputs": [
          { "input_id": "input_1" }
        ]
      }
    }
    ```
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    ```elixir
    def handle_setup(ctx, state) do
      video_update_request = %LiveCompositor.Request.UpdateVideoOutput{
        output_id: "video_output_1",
        root: {
          "type": "rescaler",
          "child": {
            "type": "input_stream",
            "input_id": "video_input_1"
          }
        }
      }
      audio_update_request = %LiveCompositor.Request.UpdateAudioOutput{
        output_id: "audio_output_1",
        inputs: [
          %{input_id: "audio_input_1"}
        ]
      }
      
      {
        [
          notify_child: {:live_compositor, video_update_request},
          notify_child: {:live_compositor, audio_update_request}
        ],
        state
      }
    end
    ```
  </TabItem>
</Tabs>
