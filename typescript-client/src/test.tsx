import { useEffect, useState } from 'react';
import LiveCompositor from './compositor';
import { Text, View } from './index';

export function TestComponent(props: any) {
  const [after, setAfter] = useState(1);
  useEffect(() => {
    const timeout = setInterval(() => {
      setAfter(after + 1);
    }, 1000);
    return () => {
      clearInterval(timeout);
    };
  });
  return (
    <View width={11}>
      <Text fontSize={after}>
        test {after ? 'after' : 'before'}
        {2}
      </Text>
      <View width={100}>
        <Text colorRgba="#FF0000FF" fontSize={50}>
          test 2
        </Text>
        {props.children}
      </View>
    </View>
  );
}

interface AppProps {
  count: number;
}

export function App(props: AppProps) {
  return (
    <View height={111}>
      {[...Array(props.count)].map((_value, index) => (
        <TestComponent key={index}>laskdjlksdfj {props.count}</TestComponent>
      ))}
      <Text fontSize={50}>lksdjf</Text>
      <View />
    </View>
  );
}

export async function test() {
  const compositor = await LiveCompositor.create();
  await compositor.registerOutput('output_1', {
    type: 'rtp_stream',
    port: 8001,
    ip: '127.0.0.1',
    transport_protocol: 'udp',
    video: {
      encoder: {
        type: 'ffmpeg_h264',
        preset: 'ultrafast',
      },
      resolution: {
        width: 1920,
        height: 1080,
      },
      root: <App count={1} />,
    },
  });
  await compositor.start();
}
