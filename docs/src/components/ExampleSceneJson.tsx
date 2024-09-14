import CodeBlock from '@theme/CodeBlock';

export default function ExampleSceneJson() {
  return (
    <CodeBlock language="typescript">
      {`{
  "type": "view",
  "children": [
    {
      "type": "shader",
      "shader_id": "replace_green_screen",
      "resolution": { "width": 1920, "height": 1080 },
      "children": [
          { "type": "input_stream", "input_id": "tv" },
          { "type": "image", "image_id": "background" }
      ]
    },
    {
      "type": "rescaler",
      "width": 640, "height": 360,
      "top": 20, "left": 20,
      "child": { 
        "type": "input_stream", "input_id": "bunny"
      }
    },
    {
      "type": "view",
      "height": 120,
      "left": 0, "bottom": 0, 
      "background_color_rgba": "#B3B3B3FF",
      "children": [
        { "type": "view" },
        {
          "type": "text", 
          "text": "LiveCompositor 😃😍",
          "font_size": 100,
          "weight": "bold",
          "color_rgba": "#000000FF",
        },
        { "type": "view" }
      ]
    }
  ]
}`}
    </CodeBlock>
  );
}
