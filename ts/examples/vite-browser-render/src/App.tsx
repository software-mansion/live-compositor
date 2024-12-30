import { useState } from 'react';
import './App.css';
import Counter from './examples/Counter';
import MP4Player from './examples/MP4Player';
import Camera from './examples/Camera';
import ScreenCapture from './examples/ScreenCapture';

const EXAMPLES = {
  counter: <Counter />,
  mp4: <MP4Player />,
  camera: <Camera />,
  screenCapture: <ScreenCapture />,
};

function App() {
  const [currentExample, setCurrentExample] = useState<keyof typeof EXAMPLES>('counter');

  return (
    <>
      <h1>Browser Renderer Examples</h1>
      <div className="examples-tabs">
        <button onClick={() => setCurrentExample('counter')}>Counter</button>
        <button onClick={() => setCurrentExample('mp4')}>MP4</button>
        <button onClick={() => setCurrentExample('camera')}>Camera</button>
        <button onClick={() => setCurrentExample('screenCapture')}>Screen Capture</button>
      </div>
      <div className="card">{EXAMPLES[currentExample]}</div>
    </>
  );
}

export default App;
