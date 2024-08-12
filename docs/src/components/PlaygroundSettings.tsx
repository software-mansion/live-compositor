import { useState } from 'react';
import ReactModal from 'react-modal';
import { InputResolutionNames, ResolutionName } from '../resolution';
import styles from './PlaygroundSettings.module.css';
import SettingsInputs from './PlaygroundSettingsInputs';

interface PlaygroundSettingsProps {
  onSubmit: () => Promise<void>;
  onChange: (input_id: string, resolution: ResolutionName) => void;
  inputResolutions: InputResolutionNames;
}

function PlaygroundSettings({ onSubmit, onChange, inputResolutions }: PlaygroundSettingsProps) {
  const [inputsSettingsModalOpen, setInputsSettingsModalOpen] = useState(false);

  return (
    <div style={{ margin: '10px' }}>
      <button
        className="button button--outline button--secondary"
        onClick={() => {
          setInputsSettingsModalOpen(true);
        }}
        style={{ margin: '10px' }}>
        Inputs settings
      </button>
      <button
        className="button button--outline button--primary"
        onClick={() => onSubmit()}
        style={{ margin: '10px' }}>
        Submit
      </button>
      <ReactModal
        isOpen={inputsSettingsModalOpen}
        onRequestClose={() => setInputsSettingsModalOpen(false)}
        overlayClassName={styles.overlay}
        className={styles.content}
        ariaHideApp={false}>
        <SettingsInputs handleSettingsUpdate={onChange} inputResolutions={inputResolutions} />
      </ReactModal>
    </div>
  );
}

export default PlaygroundSettings;
