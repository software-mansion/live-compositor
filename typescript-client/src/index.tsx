import LiveCompositor from './compositor';
import React from 'react';

import View from './components/View';
import Text from './components/Text';
import InputStream from './components/InputStream';
import Image from './components/Image';
import Rescaler from './components/Rescaler';
import WebView from './components/WebView';
import Shader from './components/Shader';
import Tiles from './components/Tiles';

export function TestComponent(props: any) {
  return (
    <View>
      <Text fontSize={50}>test {2}</Text>
      <View width={100}>
        <Text fontSize={50}>test 2</Text>
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
    <View>
      {[...Array(props.count)].map(() => (
        <TestComponent>laskdjlksdfj {props.count}</TestComponent>
      ))}
      <Text fontSize={50}>lksdjf</Text>
    </View>
  );
}

//async function test() {
//  const compositor = new LiveCompositor(<App count={1} />);
//  console.log(JSON.stringify(compositor.scene(), null, 2));
//  await compositor.start()
//
//  await compositor.api().registerInput('input_1', {
//    type: 'rtp_stream',
//    transport_protocol: 'tcp_server',
//  });
//
//  let counter = 10;
//  setInterval(() => {
//    compositor.update({ count: counter });
//    counter += 1;
//  }, 10_000);
//}

// test();

export { View, Text, InputStream, Rescaler, WebView, Image, Shader, Tiles };

export default LiveCompositor;
