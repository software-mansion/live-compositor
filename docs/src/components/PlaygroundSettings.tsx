import clsx from 'clsx';
import { useState } from 'react';
import ReactModal from 'react-modal';
import { Tooltip } from 'react-tooltip';
import { InputResolution, InputsSettings, Resolution } from '../resolution';
import styles from './PlaygroundSettings.module.css';
import SettingsInputs from './PlaygroundSettingsInputs';
import OutputResolution from './PlaygroundSettingsOutput';

interface PlaygroundSettingsProps {
  onSubmit: () => Promise<void>;
  onInputResolutionChange: (input_id: string, resolution: InputResolution) => void;
  onOutputResolutionChange: (resolution: Resolution) => void;
  inputsSettings: InputsSettings;
  readyToSubmit: boolean;
  outputResolution: Resolution;
}

export default function PlaygroundSettings({
  onSubmit,
  onInputResolutionChange,
  onOutputResolutionChange,
  inputsSettings,
  readyToSubmit,
  outputResolution,
}: PlaygroundSettingsProps) {
  const [inputsSettingsModalOpen, setInputsSettingsModalOpen] = useState(false);
  const [isResolutionValid, setIsResolutionValid] = useState(true);

  return (
    <div className={styles.settingsPanel}>
      <div className={styles.settings}>
        <div className={styles.cardsContainer}>
          <Card
            title="Inputs resolutions"
            subtitle="settings"
            onClick={() => setInputsSettingsModalOpen(true)}
          />

          <Card
            title="Images"
            subtitle="preview"
            onClick={() => setInputsSettingsModalOpen(true)}
          />

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
          setValidity={setIsResolutionValid}
        />

        <SubmitButton onSubmit={onSubmit} readyToSubmit={readyToSubmit && isResolutionValid} />
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
  readyToSubmit,
}: {
  onSubmit: () => Promise<void>;
  readyToSubmit: boolean;
}) {
  const tooltipStyle = {
    color: 'var(--ifm-font-color-base-inverse)',
    backgroundColor: 'var(--ifm-color-emphasis-700)',
  };
  return (
    <div
      data-tooltip-id={readyToSubmit ? null : 'disableSubmit'}
      data-tooltip-content={readyToSubmit ? null : 'Invalid scene provided!'}
      data-tooltip-place={readyToSubmit ? null : 'top'}>
      <button
        className={`${styles.submitButton} ${styles.hoverPrimary} ${
          readyToSubmit ? styles.submitButtonActive : styles.submitButtonInactive
        }`}
        onClick={onSubmit}>
        Submit
      </button>
      <Tooltip id="disableSubmit" style={tooltipStyle} opacity={1} />
    </div>
  );
}
