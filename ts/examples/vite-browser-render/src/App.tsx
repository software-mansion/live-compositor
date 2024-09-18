import { useState } from 'react';
import './App.css';
import Counter from './examples/Counter';
import MP4Player from './examples/MP4Player';

const EXAMPLES = {
  'counter': <Counter />,
  'mp4': <MP4Player />,
};

function App() {
  const [currentExample, setCurrentExample] = useState<keyof typeof EXAMPLES>('counter');

  return (
    <>
      <h1>Browser Renderer Examples</h1>
      <div className="examples-tabs">
        <button onClick={() => setCurrentExample('counter')}>Counter</button>
        <button onClick={() => setCurrentExample('mp4')}>MP4</button>
      </div>
      <div className="card">
        {EXAMPLES[currentExample]}
      </div>
    </>
  );
}


export default App;
