import clsx from 'clsx';
import { useState } from 'react';
import ReactModal from 'react-modal';
import { Tooltip } from 'react-tooltip';
import { InputResolution, InputsSettings, Resolution } from '../resolution';
import styles from './PlaygroundSettings.module.css';
import PlaygroundSettingsImages from './PlaygroundSettingsImages';
import SettingsInputs from './PlaygroundSettingsInputs';
import OutputResolution from './PlaygroundSettingsOutput';

interface PlaygroundSettingsProps {
  onSubmit: () => Promise<void>;
  onInputResolutionChange: (input_id: string, resolution: InputResolution) => void;
  onOutputResolutionChange: (resolution: Resolution) => void;
  inputsSettings: InputsSettings;
  sceneValidity: boolean;
  outputResolution: Resolution;
}

export default function PlaygroundSettings({
  onSubmit,
  onInputResolutionChange,
  onOutputResolutionChange,
  inputsSettings,
  sceneValidity,
  outputResolution,
}: PlaygroundSettingsProps) {
  const [inputsSettingsModalOpen, setInputsSettingsModalOpen] = useState(false);
  const [imagesModalOpen, setImagesModalOpen] = useState(false);
  const [outputResolutionValidity, setOutputResolutionValidity] = useState(true);

  return (
    <div className={styles.settingsPanel}>
      <div className={styles.settings}>
        <div className={styles.cardsContainer}>
          <Card
            title="Inputs resolutions"
            subtitle="settings"
            onClick={() => setInputsSettingsModalOpen(true)}
          />

          <Card title="Images" subtitle="preview" onClick={() => setImagesModalOpen(true)} />

          <Card
            title="Shaders"
            subtitle="preview"
            onClick={() => setInputsSettingsModalOpen(true)}
          />
        </div>
      </div>

      <div className={styles.bottomContainer}>
        <OutputResolution
          resolution={outputResolution}
          handleSettingsUpdate={onOutputResolutionChange}
          setValidity={setOutputResolutionValidity}
        />

        <SubmitButton
          onSubmit={onSubmit}
          validity={{
            scene: sceneValidity,
            outputResolution: outputResolutionValidity,
          }}
        />
      </div>

      <ReactModal
        isOpen={inputsSettingsModalOpen}
        onRequestClose={() => setInputsSettingsModalOpen(false)}
        overlayClassName={styles.modalOverlay}
        className={styles.modalContent}
        ariaHideApp={false}>
        <SettingsInputs
          handleSettingsUpdate={onInputResolutionChange}
          inputsSettings={inputsSettings}
        />
      </ReactModal>
      <ReactModal
        isOpen={imagesModalOpen}
        onRequestClose={() => setImagesModalOpen(false)}
        overlayClassName={styles.modalOverlay}
        className={styles.modalContent}
        ariaHideApp={false}>
        <PlaygroundSettingsImages />
      </ReactModal>
    </div>
  );
}

type CardProps = {
  title: string;
  subtitle: string;
  onClick: () => void;
};

function Card(props: CardProps) {
  return (
    <div className={clsx('card', styles.card, styles.hoverPrimary)} onClick={props.onClick}>
      <div className={styles.cardTitle}>{props.title}</div>
      <div className={styles.cardSubtitle}>{props.subtitle}</div>
    </div>
  );
}

function SubmitButton({
  onSubmit,
  validity,
}: {
  onSubmit: () => Promise<void>;
  validity: {
    scene: boolean;
    outputResolution: boolean;
  };
}) {
  const tooltipStyle = {
    color: 'var(--ifm-font-color-base-inverse)',
    backgroundColor: 'var(--ifm-color-emphasis-700)',
  };
  function isValid() {
    return validity.scene && validity.outputResolution;
  }
  function errorMessage() {
    if (!validity.scene) {
      return 'Invalid scene provided';
    } else if (!validity.outputResolution) {
      return 'Invalid output resolution';
    } else {
      return null;
    }
  }
  return (
    <div
      data-tooltip-id={isValid() ? null : 'disableSubmit'}
      data-tooltip-content={errorMessage()}
      data-tooltip-place={isValid() ? null : 'top'}>
      <button
        className={`${styles.submitButton} ${styles.hoverPrimary} ${
          isValid() ? styles.submitButtonActive : styles.submitButtonInactive
        }`}
        onClick={onSubmit}
        disabled={!isValid()}>
        Submit
      </button>
      <Tooltip id="disableSubmit" style={tooltipStyle} opacity={1} offset={5} />
    </div>
  );
}
