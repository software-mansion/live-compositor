import { View, Text, useInputStreams, InputStream, Tiles } from 'live-compositor';
import { Provider, useSelector } from "react-redux"

import { showInstructionsSlice, store } from "./store"

export default function App() {
  return (
    <Provider store={store}>
      <OutputScene />
    </Provider>
  )
}

function OutputScene() {
  const inputs = useInputStreams();
  const showInstructions = useSelector(showInstructionsSlice.selectors.shouldShow)
  return (
    <View>
      {showInstructions ? <Instructions /> : undefined}
      <Tiles>
        {Object.values(inputs).map((input) => (
          <InputStream key={input.inputId} inputId={input.inputId} />
        ))}
      </Tiles>
    </View >
  );
}

function Instructions() {
  return (
    <View direction="column">
      <View />
      <Text fontSize={50}>Open index.ts and get started.</Text>
      <View height={20} />
      <Text width={960} fontSize={30} wrap="word">
        This example renders static text and sends the output stream via RTP to local port
        8001. Generated code includes helpers in liveCompositorFfplayHelper.ts that display the output
        stream using ffplay, make sure to remove them for any real production use.
      </Text>
      <View height={20} />
      <Text fontSize={50}>Where to go next?</Text>
      <Text width={960} fontSize={30} wrap="word">
        - ./src/App.tsx defines content of the streams.
      </Text>
      <Text width={960} fontSize={30} wrap="word">
        - ./src/routes.ts controls HTTP API that can be used to interact with this example.
      </Text>
      <Text width={960} fontSize={30} wrap="word">
        - ./compositor.tsx exposes LiveCompositor instance that can be used to add/remove new streams/images/shader.
      </Text>
      <Text width={960} fontSize={30} wrap="word">
        - ./store.ts implements Redux store that is used for storing global state and sharing it between express API and React.
      </Text>
      <View />
    </View>
  )
}
