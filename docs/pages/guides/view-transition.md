import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';
import SimpleTransition from "./view_transtion_1.webp"
import BothStreamsTransition from "./view_transtion_2.webp"
import UnsupportedTransition from "./view_transtion_3.webp"
import InterpolationShowcaseTransition from "./view_transtion_4.webp"

# Transitions (View/Rescaler)

This guide will show a few basic examples of animated transitions on `View`/`Rescaler` components.

### Configure inputs and output

Start the compositor and configure 2 input streams and a single output stream as described in the "Simple scene"
guide in the ["Configure inputs and output"](./simple-scene.md#configure-inputs-and-output) section.

### Transition that changes the `width` of an input stream

<Tabs queryString="lang">
  <TabItem value="http" label="HTTP">
    Set initial scene for the transition:
    ```http
    POST: /api/output/output_1/update
    Content-Type: application/json

    {
      "video": {
        "root": {
          "type": "view",
          "background_color_rgba": "#4d4d4dff",
          "children": [
            {
              "type": "rescaler",
              "id": "rescaler_1",
              // highlight-start
              "width": 480,
              // highlight-end
              "child": { "type": "input_stream", "input_id": "input_1" },
            }
          ]
        }
      }
    }
    ```

    A few seconds latter update a scene with a different `width`:
    ```http
    POST: /api/output/output_1/update
    Content-Type: application/json

    {
      "video": {
        "root": {
          "type": "view",
          "background_color_rgba": "#4d4d4dff",
          "children": [
            {
              "type": "rescaler",
              "id": "rescaler_1",
              // highlight-start
              "width": 1280,
              "transition": { "duration_ms": 2000 },
              // highlight-end
              "child": { "type": "input_stream", "input_id": "input_1" },
            }
          ]
        }
      }
    }
    ```
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    Set initial scene for the transition and after few seconds update a component
    with a different `width`:
    ```elixir
    def handle_setup(ctx, state) do
      request = %LiveCompositor.Request.UpdateVideoOutput{
        output_id: "output_1",
        root: %{
          type: :view,
          background_color_rgba: "#4d4d4dff",
          children: [
            %{
              type: :rescaler,
              id: "rescaler_1",
              // highlight-start
              width: 480,
              // highlight-end
              child: %{ type: :input_stream, input_id: :input_1 },
            }
          ]
        }
      }
      Process.send_after(self(), :start_transition, 2000)
      {[notify_child: {:live_compositor, request}], state}
    end

    def handle_info(:start_transition, _ctx, state)
      request = %LiveCompositor.Request.UpdateVideoOutput{
        output_id: "output_1",
        root: %{
          type: :view,
          background_color_rgba: "#4d4d4dff",
          children: [
            %{
              type: :rescaler,
              id: "rescaler_1",
              // highlight-start
              width: 1280,
              transition: %{ duration_ms: 2000 },
              // highlight-end
              child: %{ type: :input_stream, input_id: :input_1 },
            }
          ]
        }
      }
      {[notify_child: {:live_compositor, request}], state}
    end
    ```
  </TabItem>
</Tabs>

In the first update request, you can see that the rescaler has a width of 480, and in the second one, it is changed
to 1280 and `transition.duration_ms: 2000` was added.

The component must have the same `"id"` in both the initial state and the update that starts the
transition, otherwise it will switch immediately to the new state without a transition.

<div style={{textAlign: 'center'}}>
    <img src={SimpleTransition} style={{ width: 600 }} />
    Output stream
</div>

### Transition on one of the sibling components

In the above scenario you saw how transition on a single component behaves, but let's see what happens with
components that are not a part of the transition, but their size and position still depend on other components.

Add a second input stream wrapped with `Rescaler`, but without any transition options.

<Tabs queryString="lang">
  <TabItem value="http" label="HTTP">
    ```http
    POST: /api/output/output_1/update
    Content-Type: application/json

    {
      "video": {
        "root": {
          "type": "view",
          "background_color_rgba": "#4d4d4dff",
          "children": [
            {
              "type": "rescaler",
              "id": "rescaler_1",
              // highlight-start
              "width": 480,
              // highlight-end
              "child": { "type": "input_stream", "input_id": "input_1" },
            },
            {
              "type": "rescaler",
              "child": { "type": "input_stream", "input_id": "input_2" },
            }
          ]
        }
      }
    }
    ```

    Update a scene with a different `width`:
    ```http
    POST: /api/output/output_1/update
    Content-Type: application/json

    {
      "video": {
        "root": {
          "type": "view",
          "background_color_rgba": "#4d4d4dff",
          "children": [
            {
              "type": "rescaler",
              "id": "rescaler_1",
              // highlight-start
              "width": 1280,
              "transition": { "duration_ms": 2000 },
              // highlight-end
              "child": { "type": "input_stream", "input_id": "input_1" },
            },
            {
              "type": "rescaler",
              "child": { "type": "input_stream", "input_id": "input_2" },
            }
          ]
        }
      }
    }
    ```
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    ```elixir
    def handle_setup(ctx, state) do
      request = %LiveCompositor.Request.UpdateVideoOutput{
        output_id: "output_1",
        root: %{
          type: :view,
          background_color_rgba: "#4d4d4dff",
          children: [
            %{
              type: :rescaler,
              id: "rescaler_1",
              // highlight-start
              width: 480,
              // highlight-end
              child: %{ type: :input_stream, input_id: :input_1 },
            },
            %{
              type: :rescaler,
              child: %{ type: :input_stream, input_id: :input_2 },
            }
          ]
        }
      }
      Process.send_after(self(), :start_transition, 2000)
      {[notify_child: {:live_compositor, request}], state}
    end

    def handle_info(:start_transition, _ctx, state)
      request = %LiveCompositor.Request.UpdateVideoOutput{
        output_id: "output_1",
        root: %{
          type: :view,
          background_color_rgba: "#4d4d4dff",
          children: [
            %{
              type: :rescaler,
              id: "rescaler_1",
              // highlight-start
              width: 1280,
              transition: %{ duration_ms: 2000 },
              // highlight-end
              child: %{ type: :input_stream, input_id: :input_1 },
            },
            %{
              type: :rescaler,
              child: %{ type: :input_stream, input_id: :input_2 },
            }
          ]
        }
      }
      {[notify_child: {:live_compositor, request}], state}
    end
    ```
  </TabItem>
</Tabs>


<div style={{textAlign: 'center'}}>
    <img src={BothStreamsTransition} style={{ width: 600 }} />
    Output stream
</div>


### Transition between different modes

Currently, a state before the transition and after needs to use the same type of configuration. In particular:
- It is not possible to transition a component between static and absolute positioning.
- It is not possible to transition a component between using `top` and `bottom` fields (the same for `left`/`right`).
- It is not possible to transition a component with known `width`/`height` to a state with dynamic `width`/`height` based
on the parent layout.

Let's try the same example as in the first scenario with a single input, but instead, change the `Rescaler` component to be absolutely positioned in the second update.

<Tabs queryString="lang">
  <TabItem value="http" label="HTTP">
    ```http
    POST: /api/output/output_1/update
    Content-Type: application/json

    {
      "video": {
        "root": {
          "type": "view",
          "background_color_rgba": "#4d4d4dff",
          "children": [
            {
              "type": "rescaler",
              "id": "rescaler_1",
              // highlight-start
              "width": 480,
              // highlight-end
              "child": { "type": "input_stream", "input_id": "input_1" },
            }
          ]
        }
      }
    }
    ```

    ```http
    POST: /api/output/output_1/update
    Content-Type: application/json

    {
      "video": {
        "root": {
          "type": "view",
          "background_color_rgba": "#4d4d4dff",
          "children": [
            {
              "type": "rescaler",
              "id": "rescaler_1",
              // highlight-start
              "width": 1280,
              "top": 0,
              "left": 0,
              "transition": { "duration_ms": 2000 },
              // highlight-end
              "child": { "type": "input_stream", "input_id": "input_1" },
            }
          ]
        }
      }
    }
    ```
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    ```elixir
    def handle_setup(ctx, state) do
      request = %LiveCompositor.Request.UpdateVideoOutput{
        output_id: "output_1",
        root: %{
          type: :view,
          background_color_rgba: "#4d4d4dff",
          children: [
            %{
              type: :rescaler,
              id: "rescaler_1",
              // highlight-start
              width: 480,
              // highlight-end
              child: %{ type: :input_stream, input_id: :input_1 },
            }
          ]
        }
      }
      Process.send_after(self(), :start_transition, 2000)
      {[notify_child: {:live_compositor, request}], state}
    end

    def handle_info(:start_transition, _ctx, state)
      request = %LiveCompositor.Request.UpdateVideoOutput{
        output_id: "output_1",
        root: %{
          type: :view,
          background_color_rgba: "#4d4d4dff",
          children: [
            %{
              type: :rescaler,
              id: "rescaler_1",
              // highlight-start
              width: 1280,
              top: 0,
              left: 0,
              transition: %{ duration_ms: 2000 },
              // highlight-end
              child: %{ type: :input_stream, input_id: :input_1 },
            }
          ]
        }
      }
      {[notify_child: {:live_compositor, request}], state}
    end
    ```
  </TabItem>
</Tabs>

As you can see on the resulting stream, the transition did not happen because the `Rescaler` component
in the initial scene was using static positioning and after the update it was positioned absolutely.

<div style={{textAlign: 'center'}}>
    <img src={UnsupportedTransition} style={{ width: 600 }} />
    Output stream
</div>

### Different interpolation functions

All of the above examples use default linear interpolation, but there are also a few other
modes available.

<Tabs queryString="lang">
  <TabItem value="http" label="HTTP">
    ```http
    POST: /api/output/output_1/update
    Content-Type: application/json

    {
      "video": {
        "root": {
          "type": "view",
          "background_color_rgba": "#4d4d4dff",
          "children": [
            {
              "type": "rescaler",
              "id": "rescaler_1",
              "width": 320, "height": 180, "top": 0, "left": 0,
              "child": { "type": "input_stream", "input_id": "input_1" },
            },
            {
              "type": "rescaler",
              "id": "rescaler_2",
              "width": 320, "height": 180, "top": 0, "left": 320,
              "child": { "type": "input_stream", "input_id": "input_2" },
            },
            {
              "type": "rescaler",
              "id": "rescaler_3",
              "width": 320, "height": 180, "top": 0, "left": 640,
              "child": { "type": "input_stream", "input_id": "input_3" },
            },
            {
              "type": "rescaler",
              "id": "rescaler_4",
              "width": 320, "height": 180, "top": 0, "left": 960,
              "child": { "type": "input_stream", "input_id": "input_4" },
            },
          ]
        }
      }
    }
    ```

    ```http
    POST: /api/output/output_1/update
    Content-Type: application/json

    {
      "video": {
        "root": {
          "type": "view",
          "background_color_rgba": "#4d4d4dff",
          "children": [
            {
              "type": "rescaler",
              "id": "rescaler_1",
              "width": 320, "height": 180, "top": 540, "left": 0,
              "child": { "type": "input_stream", "input_id": "input_1" },
              "transition": { "duration_ms": 2000 },
            },
            {
              "type": "rescaler",
              "id": "rescaler_2",
              "width": 320, "height": 180, "top": 540, "left": 320,
              "child": { "type": "input_stream", "input_id": "input_2" },
              "transition": {
                "duration_ms": 2000, "easing_function": {"function_name": "bounce"}
              },
            },
            {
              "type": "rescaler",
              "id": "rescaler_3",
              "width": 320, "height": 180, "top": 540, "left": 640,
              "child": { "type": "input_stream", "input_id": "input_3" },
              "transition": {
                "duration_ms": 2000,
                "easing_function": {
                    "function_name": "cubic_bezier",
                    "points": [0.65, 0, 0.35, 1]
                }
              }
            },
            {
              "type": "rescaler",
              "id": "rescaler_4",
              "width": 320, "height": 180, "top": 540, "left": 960,
              "child": { "type": "input_stream", "input_id": "input_4" },
              "transition": {
                "duration_ms": 2000,
                "easing_function": {
                  "function_name": "cubic_bezier",
                  "points": [0.33, 1, 0.68, 1]
                }
              }
            }
          ]
        }
      }
    }
    ```
  </TabItem>
  <TabItem value="membrane" label="Membrane Framework">
    ```elixir
    def handle_setup(ctx, state) do
      request = %LiveCompositor.Request.UpdateVideoOutput{
        output_id: "output_1",
        root: %{
          type: :view,
          background_color_rgba: "#4d4d4dff",
          children: [
            %{
              type: :rescaler,
              id: "rescaler_1",
              width: 320, height: 180, top: 0, left: 0,
              child: %{ type: :input_stream, input_id: :input_1 },
            },
            %{
              type: :rescaler,
              id: "rescaler_2",
              width: 320, height: 180, top: 0, left: 320,
              child: %{ type: :input_stream, input_id: :input_2 },
            },
            %{
              type: :rescaler,
              id: "rescaler_3",
              width: 320, height: 180, top: 0, left: 640,
              child: %{ type: :input_stream, input_id: :input_3 },
            },
            %{
              type: :rescaler,
              id: "rescaler_4",
              width: 320, height: 180, top: 0, left: 960,
              child: %{ type: :input_stream, input_id: :input_4 },
            }
          ]
        }
      }
      Process.send_after(self(), :start_transition, 2000)
      {[notify_child: {:live_compositor, request}], state}
    end

    def handle_info(:start_transition, _ctx, state)
      request = %LiveCompositor.Request.UpdateVideoOutput{
        output_id: "output_1",
        root: %{
          type: :view,
          background_color_rgba: "#4d4d4dff",
          children: [
            %{
              type: :rescaler,
              id: "rescaler_1",
              width: 320, height: 180, top: 0, left: 0,
              child: %{ type: :input_stream, input_id: :input_1 },
              transition: %{ duration_ms: 2000 },
            },
            %{
              type: :rescaler,
              id: "rescaler_2",
              width: 320, height: 180, top: 0, left: 320,
              child: %{ type: :input_stream, input_id: :input_2 },
              transition: %{
                duration_ms: 2000
                easing_function: %{ function_name: :bounce}
              },
            },
            %{
              type: :rescaler,
              id: "rescaler_3",
              width: 320, height: 180, top: 0, left: 640,
              child: %{ type: :input_stream, input_id: :input_3 },
              transition: %{
                duration_ms: 2000
                easing_function: %{
                  function_name: :cubic_bezier,
                  points: [0.65, 0, 0.35, 1]
                }
              }
            },
            %{
              type: :rescaler,
              id: "rescaler_4",
              width: 320, height: 180, top: 0, left: 960,
              child: %{ type: :input_stream, input_id: :input_4 },
              transition: %{
                duration_ms: 2000
                easing_function: %{
                  function_name: :cubic_bezier,
                  points: [0.33, 1, 0.68, 1]
                }
              }
            }
          ]
        }
      }
      {[notify_child: {:live_compositor, request}], state}
    end
    ```
  </TabItem>
</Tabs>


<div style={{textAlign: 'center'}}>
    <img src={InterpolationShowcaseTransition} style={{ width: 600 }} />
    Output stream
</div>

- `Input 1` - Linear transition
- `Input 2` - Bounce transition
- `Input 3` - Cubic Bézier transition with `[0.65, 0, 0.35, 1]` points ([`easeInOutCubic`](https://easings.net/#easeInOutCubic))
- `Input 4` - Cubic Bézier transition with `[0.33, 1, 0.68, 1]` points ([`easeOutCubic`](https://easings.net/#easeOutCubic))

Check out other popular Cubic Bézier curves on [https://easings.net](https://easings.net).
