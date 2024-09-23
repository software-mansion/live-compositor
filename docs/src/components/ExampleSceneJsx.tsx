import CodeBlock from '@theme/CodeBlock';

const JSX_CODE = `import {
  InputStream,
  Image,
  Rescaler
  Shader,
  Text,
  View,
} from "live-compositor"

function Example() {
  return (
    <View>
      <Shader
        shaderId="replace_green_screen"
        resolution={{ width: 1920, height: 1080 }}>
        <InputStream inputId="tv" />
        <Image imageId="background" />
      </Shader>
      <Rescaler top={20} left={20} width={640} height={360}>
        <InputStream inputId="bunny" />
      </Rescaler>
      <View bottom={0} left={0} height={120}>
        <View />
        <Text fontSize={100} weight="bold" color="#000000">
          LiveCompositor 😃😍
        </Text>
        <View />
      </View>
    </View>
  );
}
`;

export default function ExampleSceneJsx() {
  return <CodeBlock language="tsx">{JSX_CODE}</CodeBlock>;
}
