import { View, Text, useInputStreams, InputStream, Tiles } from 'live-compositor';

import { store } from './store';
import { useStore } from 'zustand';

export default function App() {
  return <OutputScene />;
}

function OutputScene() {
  const inputs = useInputStreams();
  const showInstructions = useStore(store, state => state.shouldShowInstructions);

  return (
    <View>
      {showInstructions ? <Instructions /> : undefined}
      <Tiles>
        {Object.values(inputs).map(input => (
          <InputStream key={input.inputId} inputId={input.inputId} />
        ))}
      </Tiles>
    </View>
  );
}

function Instructions() {
  return (
    <View style={{ direction: 'column' }}>
      <View />
      <Text style={{ fontSize: 50 }}>Open index.ts and get started.</Text>
      <View style={{ height: 20 }} />
      <Text style={{ width: 960, fontSize: 30, wrap: 'word' }}>
        This example renders static text and sends the output stream via RTP to local port 8001.
        Generated code includes helpers in liveCompositorFfplayHelper.ts that display the output
        stream using ffplay, make sure to remove them for any real production use.
      </Text>
      <View style={{ height: 20 }} />
      <Text style={{ fontSize: 50 }}>Where to go next?</Text>
      <Text style={{ width: 960, fontSize: 30, wrap: 'word' }}>
        - ./src/App.tsx defines content of the streams.
      </Text>
      <Text style={{ width: 960, fontSize: 30, wrap: 'word' }}>
        - ./src/routes.ts controls HTTP API that can be used to interact with this example.
      </Text>
      <Text style={{ width: 960, fontSize: 30, wrap: 'word' }}>
        - ./compositor.tsx exposes LiveCompositor instance that can be used to add/remove new
        streams/images/shader.
      </Text>
      <Text style={{ width: 960, fontSize: 30, wrap: 'word' }}>
        - ./store.ts implements global store using Zustand, enabling express API and React to share
        common settings.
      </Text>
      <View />
    </View>
  );
}
