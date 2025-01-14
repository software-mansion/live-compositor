import { useState } from 'react';
import './App.css';
import Counter from './examples/Counter';
import SimpleMp4Example from './examples/SimpleMp4Example';
import MultipleCompositors from './examples/MultipleCompositors';
import { setWasmBundleUrl } from '@live-compositor/web-wasm';

setWasmBundleUrl('assets/live-compositor.wasm');

function App() {
  const EXAMPLES = {
    counter: <Counter />,
    simpleMp4: <SimpleMp4Example />,
    multipleCompositors: <MultipleCompositors />,
    home: <Home />,
  };
  const [currentExample, setCurrentExample] = useState<keyof typeof EXAMPLES>('home');

  return (
    <>
      <h1>Examples</h1>
      <div className="examples-tabs">
        <button onClick={() => setCurrentExample('home')}>Home</button>
        <button onClick={() => setCurrentExample('simpleMp4')}>Simple MP4</button>
        <button onClick={() => setCurrentExample('multipleCompositors')}>
          Multiple LiveCompositor instances
        </button>
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
        <code>Simple Mp4</code> - Take MP4 file as an input and render output on canvas
      </li>
      <li>
        <code>Multiple LiveCompositor instances</code> - Runs multiple LiveCompositor instances at
        the same time.
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
