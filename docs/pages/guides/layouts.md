import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';
import LayoutsEmpty from "./assets/layouts_1.webp"
import LayoutsOverflow from "./assets/layouts_2.webp"
import LayoutsFitted from "./assets/layouts_3.webp"
import LayoutsBothInputs from "./assets/layouts_4.webp"
import LayoutsAbsolutePosition from "./assets/layouts_5.webp"

# Layouts

This guide will explain how to create simple scene that is combining input streams in a simple layout into single output stream.

### Configure inputs and output

Start the compositor and configure 2 input streams and a single output stream as described in the "Simple scene"
guide in the ["Configure inputs and output"](./quick-start.md#configure-inputs-and-output) section.

After configuration you should see the following output:

<div style={{textAlign: 'center'}}>
    <img src={LayoutsEmpty} style={{ width: 600 }} />
    Output stream
</div>

## Update scene to show an input

Update output to render a [`View`](../api/components/View.md) component with an [`InputStream`](../api/components/InputStream.md) as its child.

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
            { "type": "input_stream", "input_id": "input_1" },
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
            %{ type: :input_stream, input_id: :input_1 }
          ]
        }
      }
      {[notify_child: {:live_compositor, request}], state}
    end
    ```
  </TabItem>
</Tabs>

The input stream in the example has a resolution `1920x1080` and it is rendered on the `1270x720` output. As a result only part of the stream is visible.

<div style={{textAlign: 'center'}}>
    <img src={LayoutsOverflow} style={{ width: 600 }} />
    Output stream
</div>

## Resize input stream to fit inside the output

Wrap an [`InputStream`](../api/components/InputStream.md) component with a [`Rescaler`](../api/components/Rescaler.md).

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

Input stream now fully fits inside the output.

<div style={{textAlign: 'center'}}>
    <img src={LayoutsFitted} style={{ width: 600 }} />
    Output stream
</div>

:::note
The same effect (for single input) could be achieved by either:
- Setting `InputStream` as a root directly. It would only work if aspect ratio of input and output is the same.
- Replacing `View` component with a `Rescaler`.
:::

## Show both inputs side by side

Add another [`InputStream`](../api/components/InputStream.md) wrapped with [`Rescaler`](../api/components/Rescaler.md).

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

By default, a `View` component positions its children next to each other in a row. Each child without a defined width or
height fills available space inside the parent component. To place them in a column, set a `direction: "column"` option.

In an example below we can see that:
- `View` has 2 children components with unspecified dimensions, so they will divide available width exactly in half.
- Each `Rescaler` component has a size `640x720` (half of `1280x720`), but it needs to fit an input stream with `16:9` aspect ratio.

<div style={{textAlign: 'center'}}>
    <img src={LayoutsBothInputs} style={{ width: 600 }} />
    Output stream
</div>

## Place one of the inputs in the top right corner

Specify `width` and `height` of one of the `Rescaler` components and position it using `top`/`right` options in the corner.

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
              "child": { "type": "input_stream", "input_id": "input_1" },
            },
            {
              "type": "rescaler",
              "width": 320,
              "height": 180,
              "top": 20,
              "right": 20,
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
              child: %{ type: :input_stream, input_id: :input_1 },
            },
            %{
              type: :rescaler,
              width: 320,
              height: 180,
              top: 20,
              right: 20,
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

When you specify `top`/`right` options on the `Rescaler` component, the `View` component does not take that component
into account when calculating the row layout of its children. See [absolute positioning](../api/components/View.md#absolute-positioning) to learn more.

As a result:
- The first child extends to the full width of a parent.
- The second component takes the specified size, and it is positioned in the top-right corner (20px from border).

<div style={{textAlign: 'center'}}>
    <img src={LayoutsAbsolutePosition} style={{ width: 600 }} />
    Output stream
</div>
