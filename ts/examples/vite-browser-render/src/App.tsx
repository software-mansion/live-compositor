import { useState } from 'react';
import './App.css';
import Counter from './examples/Counter';
import InputMp4Example from './examples/InputMp4Example';
import ComponentMp4Example from './examples/ComponentMp4Example';
import MultipleCompositors from './examples/MultipleCompositors';
import Camera from './examples/CameraExample';
import ScreenCapture from './examples/ScreenCaptureExample';
import { setWasmBundleUrl } from '@live-compositor/web-wasm';
import WhipExample from './examples/WhipExample';

setWasmBundleUrl('/assets/live-compositor.wasm');

function App() {
  const EXAMPLES = {
    counter: <Counter />,
    inputMp4: <InputMp4Example />,
    componentMp4: <ComponentMp4Example />,
    whip: <WhipExample />,
    multipleCompositors: <MultipleCompositors />,
    camera: <Camera />,
    screenCapture: <ScreenCapture />,
    home: <Home />,
  };
  const [currentExample, setCurrentExample] = useState<keyof typeof EXAMPLES>('home');

  return (
    <>
      <h1>Examples</h1>
      <div className="examples-tabs">
        <button onClick={() => setCurrentExample('home')}>Home</button>
        <button onClick={() => setCurrentExample('whip')}>WHIP</button>
        <button onClick={() => setCurrentExample('inputMp4')}>Input Stream MP4</button>
        <button onClick={() => setCurrentExample('componentMp4')}>Component MP4</button>
        <button onClick={() => setCurrentExample('multipleCompositors')}>
          Multiple LiveCompositor instances
        </button>
        <button onClick={() => setCurrentExample('camera')}>Camera</button>
        <button onClick={() => setCurrentExample('screenCapture')}>Screen Capture</button>
        <button onClick={() => setCurrentExample('counter')}>Counter</button>
      </div>
      <div className="card">{EXAMPLES[currentExample]}</div>
    </>
  );
}

function Home() {
  return (
    <div style={{ textAlign: 'left' }}>
      <h2>Packages:</h2>
      <h3>
        <code>@live-compositor/web-wasm</code> - LiveCompositor in the browser
      </h3>
      <li>
        <code>WHIP</code> - Streams Mp4 file to Twitch. Add{' '}
        <code>?twitchKey=mytwitchstreamkey</code> query param with your Twitch stream key to stream
        it yourself.
      </li>
      <li>
        <code>Input Stream Mp4</code> - Register MP4 file as an input stream and render output on
        canvas.
      </li>
      <li>
        <code>Component Mp4</code> - Add 2 MP4 component (one after the other) to the scene and
        render output on canvas.
      </li>
      <li>
        <code>Multiple LiveCompositor instances</code> - Runs multiple LiveCompositor instances at
        the same time.
      </li>
      <li>
        <code>Camera</code> - Use webcam as an input and render output on canvas.
      </li>
      <li>
        <code>Screen Capture</code> - Use screen capture as an input and render output on canvas.
      </li>
      <h3>
        <code>@live-compositor/browser-render</code> - Rendering engine from LiveCompositor
      </h3>
      <li>
        <code>Counter</code> - Render a GIF + counter trigged by user(with a button).
      </li>
    </div>
  );
}

export default App;
