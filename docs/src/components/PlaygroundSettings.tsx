import clsx from 'clsx';
import { useState } from 'react';
import ReactModal from 'react-modal';
import { Tooltip } from 'react-tooltip';
import { InputResolution, InputsSettings, Resolution } from '../resolution';
import styles from './PlaygroundSettings.module.css';
import PlaygroundSettingsImages from './PlaygroundSettingsImages';
import SettingsInputs from './PlaygroundSettingsInputs';
import OutputResolution from './PlaygroundSettingsOutput';
import PlaygroundSettingsShaders from './PlaygroundSettingsShaders';
import PlaygroundSettingsExamples from './PlaygroundSettingsExamples';

type ModalContent = 'inputs' | 'images' | 'shaders' | 'examples';

interface PlaygroundSettingsProps {
  onSubmit: () => Promise<void>;
  onInputResolutionChange: (input_id: string, resolution: InputResolution) => void;
  onOutputResolutionChange: (resolution: Resolution) => void;
  setExample: (content: object | Error) => void;
  inputsSettings: InputsSettings;
  sceneValidity: boolean;
  outputResolution: Resolution;
  isLoading: boolean;
}

export default function PlaygroundSettings({
  onSubmit,
  onInputResolutionChange,
  onOutputResolutionChange,
  setExample,
  inputsSettings,
  sceneValidity,
  outputResolution,
  isLoading,
}: PlaygroundSettingsProps) {
  const [modalContent, setModalContent] = useState<ModalContent | null>(null);
  const [outputResolutionValidity, setOutputResolutionValidity] = useState<boolean>(true);

  function closeModal() {
    setModalContent(null);
  }
  const modalContentElement =
    modalContent === 'inputs' ? (
      <SettingsInputs
        handleSettingsUpdate={onInputResolutionChange}
        inputsSettings={inputsSettings}
      />
    ) : modalContent === 'images' ? (
      <PlaygroundSettingsImages />
    ) : modalContent === 'examples' ? (
      <PlaygroundSettingsExamples closeModal={closeModal} setExample={setExample} />
    ) : (
      <PlaygroundSettingsShaders />
    );

  return (
    <div className={styles.settingsPanel}>
      <div className={styles.settings}>
        <div className={styles.cardsContainer}>
          <Card
            title="Inputs"
            subtitle="Configure resolution for example inputs"
            onClick={() => setModalContent('inputs')}
          />
          <Card
            title="Images"
            subtitle="Check out available images and how to use them"
            onClick={() => setModalContent('images')}
          />
          <Card
            title="Shaders"
            subtitle="Check out available shaders and how to use them"
            onClick={() => setModalContent('shaders')}
          />
          <Card
            title="Examples"
            subtitle="Check out some example scenes"
            onClick={() => setModalContent('examples')}
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
          isLoading={isLoading}
          validity={{
            scene: sceneValidity,
            outputResolution: outputResolutionValidity,
          }}
        />
      </div>

      <ReactModal
        isOpen={modalContent !== null}
        onRequestClose={() => setModalContent(null)}
        overlayClassName={styles.modalOverlay}
        className={styles.modalContent}
        ariaHideApp={false}>
        {modalContentElement}
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
  isLoading,
  validity,
}: {
  onSubmit: () => Promise<void>;
  isLoading: boolean;
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
    return validity.scene && validity.outputResolution && !isLoading;
  }
  function errorMessage() {
    if (!validity.scene) {
      return 'Invalid scene provided';
    } else if (!validity.outputResolution) {
      return 'Invalid output resolution';
    } else if (isLoading) {
      return 'Loading...';
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
