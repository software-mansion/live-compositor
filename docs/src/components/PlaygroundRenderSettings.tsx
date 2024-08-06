import { useState } from 'react';
import PlaygroundRenderSettingsInputs from './PlaygroundRenderSettingsInputs';

interface PlaygroundRenderSettingsProps {
  onSubmit: () => Promise<void>;
  onChange: (string, Resolution) => void;
}

function PlaygroundRenderSettings({ onSubmit, onChange }: PlaygroundRenderSettingsProps) {
  const [inputsSettingsModalOpen, setInputsSettingsModalOpen] = useState(false);

  return (
    <div style={{ margin: '10px' }}>
      <div>
        <button
          className="button button--outline button--secondary"
          onClick={() => {
            setInputsSettingsModalOpen(true);
          }}
          style={{ margin: '10px' }}>
          Inputs settings
        </button>
        <PlaygroundRenderSettingsInputs
          isOpen={inputsSettingsModalOpen}
          closeModal={() => setInputsSettingsModalOpen(false)}
          handleSettingsUpdate={onChange}
        />
        <button
          className="button button--outline button--primary"
          onClick={() => onSubmit()}
          style={{ margin: '10px' }}>
          Submit
        </button>
      </div>
    </div>
  );
}

export default PlaygroundRenderSettings;
