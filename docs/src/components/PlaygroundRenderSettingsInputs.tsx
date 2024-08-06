import { useState } from 'react';
import ReactModal from 'react-modal';
import { AVAILABLE_RESOLUTIONS, Resolution } from '../pages/Resolution';
import styles from './PlaygroundRenderSettingsInputs.module.css';

enum ResolutionName {
  Resoultion1080x1920 = 'Resoultion1080x1920',
  Resoultion1920x1080 = 'Resoultion1920x1080',
  Resoultion854x480 = 'Resoultion854x480',
  Resoultion480x854 = 'Resoultion480x854',
  Resoultion1440x1080 = 'Resoultion1440x1080',
  Resoultion1080x1440 = 'Resoultion1080x1440',
}

export type InputResolutionsNames = {
  input_1: ResolutionName;
  input_2: ResolutionName;
  input_3: ResolutionName;
  input_4: ResolutionName;
  input_5: ResolutionName;
  input_6: ResolutionName;
};

function nameToResolution(name: ResolutionName): Resolution {
  switch (name) {
    case ResolutionName.Resoultion1920x1080:
      return AVAILABLE_RESOLUTIONS.Resoultion1920x1080;
    case ResolutionName.Resoultion1080x1920:
      return AVAILABLE_RESOLUTIONS.Resoultion1080x1920;
    case ResolutionName.Resoultion1440x1080:
      return AVAILABLE_RESOLUTIONS.Resoultion1440x1080;
    case ResolutionName.Resoultion1080x1440:
      return AVAILABLE_RESOLUTIONS.Resoultion1080x1440;
    case ResolutionName.Resoultion854x480:
      return AVAILABLE_RESOLUTIONS.Resoultion854x480;
    case ResolutionName.Resoultion480x854:
      return AVAILABLE_RESOLUTIONS.Resoultion480x854;
  }
}

interface PlaygroundSettingsInputsProps {
  isOpen: boolean;
  closeModal: () => void;
  handleSettingsUpdate: (string, Resolution) => void;
}

export default function PlaygroundRenderSettingsInputs({
  isOpen,
  closeModal,
  handleSettingsUpdate,
}: PlaygroundSettingsInputsProps) {
  const [selectedValues, setSelectedValues] = useState<InputResolutionsNames>({
    input_1: ResolutionName.Resoultion1920x1080,
    input_2: ResolutionName.Resoultion1920x1080,
    input_3: ResolutionName.Resoultion1920x1080,
    input_4: ResolutionName.Resoultion1920x1080,
    input_5: ResolutionName.Resoultion1920x1080,
    input_6: ResolutionName.Resoultion1920x1080,
  });

  function handleChange(event, inputId: string) {
    setSelectedValues({
      ...selectedValues,
      [inputId]: event.target.value,
    });
    handleSettingsUpdate(inputId, nameToResolution(event.target.value));
  }

  return (
    <ReactModal
      isOpen={isOpen}
      onRequestClose={closeModal}
      className={styles.content}
      overlayClassName={styles.overlay}
      ariaHideApp={false}>
      {Object.keys(selectedValues).map(inputId => (
        <InputResolutionSelect
          inputName={inputId}
          selectedValue={selectedValues[inputId]}
          handleChange={event => handleChange(event, inputId)}
          key={inputId}
        />
      ))}
      <div className={styles.controlButtonsContainer}>
        <button
          onClick={closeModal}
          className={`${styles.controlButton} button button--block button--outline button--primary`}>
          Close
        </button>
      </div>
    </ReactModal>
  );
}

interface InputResolutionSelectProps {
  inputName: string;
  selectedValue: ResolutionName;
  handleChange: (Event) => void;
}

function InputResolutionSelect({
  inputName,
  selectedValue,
  handleChange,
}: InputResolutionSelectProps) {
  return (
    <div className={styles.inputSelector}>
      <label className={styles.inputSelector_label}>{inputName}</label>
      <select onChange={handleChange} value={selectedValue} className={styles.inputSelector_select}>
        <option value={ResolutionName.Resoultion1920x1080}>[16:9] 1920x1080</option>
        <option value={ResolutionName.Resoultion1080x1920}>[9:16] 1080x1920</option>
        <option value={ResolutionName.Resoultion854x480}>[16:9] 854x480</option>
        <option value={ResolutionName.Resoultion480x854}>[9:16] 480x854</option>
        <option value={ResolutionName.Resoultion1440x1080}>[4:3] 1440x1080</option>
        <option value={ResolutionName.Resoultion1080x1440}>[3:4] 1080x1440</option>
      </select>
    </div>
  );
}
