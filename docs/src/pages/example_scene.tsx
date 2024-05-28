import CodeBlock from '@theme/CodeBlock';

export default function ExampleScene() {
  return (
    <CodeBlock language="jsx">
      {`{
  "type": "view",
  "children": [
    {
      "type": "rescaler",
      "child": {
        "type": "shader",
        "shader_id": "remove_green_screen",
        "resolution": { "width": 1920, "height": 1080 },
        "children": [
            { "type": "input_stream", "input_id": "tv" },
            { "type": "input_stream", "input_id": "bg" }
        ]
      }
    },
    {
      "type": "rescaler",
      "width": 800, "height": 450,
      "top": 20, "left": 20,
      "child": { 
        "type": "input_stream", "input_id": "bunny"
      }
    },
    {
      "type": "view",
      "left": 0, "bottom": 0, "height": 150,
      "background_color_rgba": "#FF0000FF",
      "children": [{
        "type": "text", 
        "text": "LiveCompositor 😃😍",
        "font_size": 100, "weight": "bold",
        "color_rgba": "#675BF1FF",
      }]
    }
  ]
}`}
    </CodeBlock>
  );
}
