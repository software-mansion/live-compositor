import CodeBlock from '@theme/CodeBlock';

export default function ExampleScene() {
  return (
    <CodeBlock language="typescript">
      {`{
  "type": "view",
  "children": [
    {
      "type": "shader",
      "shader_id": "remove_green_screen",
      "resolution": { "width": 1920, "height": 1080 },
      "children": [
          { "type": "input_stream", "input_id": "tv" },
          { "type": "image", "image_id": "background" }
      ]
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
      "height": 150,
      "left": 0, "bottom": 0, 
      "background_color_rgba": "#FFFF00FF",
      "children": [{
        "type": "text", 
        "text": "LiveCompositor 😃😍",
        "font_size": 100,
        "weight": "bold",
        "color_rgba": "#000000FF",
      }]
    }
  ]
}`}
    </CodeBlock>
  );
}
