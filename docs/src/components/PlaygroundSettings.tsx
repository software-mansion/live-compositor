import clsx from 'clsx';
import { useState } from 'react';
import ReactModal from 'react-modal';
import { Tooltip } from 'react-tooltip';
import { InputResolutionNames, ResolutionName } from '../resolution';
import styles from './PlaygroundSettings.module.css';
import SettingsInputs from './PlaygroundSettingsInputs';

interface PlaygroundSettingsProps {
  onSubmit: () => Promise<void>;
  onChange: (input_id: string, resolution: ResolutionName) => void;
  inputResolutions: InputResolutionNames;
  readyToSubmit: boolean;
}

export default function PlaygroundSettings({
  onSubmit,
  onChange,
  inputResolutions,
  readyToSubmit,
}: PlaygroundSettingsProps) {
  const [inputsSettingsModalOpen, setInputsSettingsModalOpen] = useState(false);

  return (
    <div className={styles.settingsPanel}>
      <div className={styles.cardsContainer}>
        <UseCaseCard
          title="Inputs resolutions"
          subtitle="settings"
          onClick={() => setInputsSettingsModalOpen(true)}
        />

        {/* <UseCaseCard
          title="Images"
          subtitle="preview"
          onClick={() => setInputsSettingsModalOpen(true)}
        /> */}

        {/* <UseCaseCard
          title="Shaders"
          subtitle="preview"
          onClick={() => setInputsSettingsModalOpen(true)}
        /> */}
      </div>
      {/* <OutputResolution /> */}

      <div className={styles.submitButtonContainer}>
        <SubmitButton onSubmit={onSubmit} readyToSubmit={readyToSubmit} />
      </div>

      <ReactModal
        isOpen={inputsSettingsModalOpen}
        onRequestClose={() => setInputsSettingsModalOpen(false)}
        overlayClassName={styles.modalOverlay}
        className={styles.modalContent}
        ariaHideApp={false}>
        <SettingsInputs handleSettingsUpdate={onChange} inputResolutions={inputResolutions} />
      </ReactModal>
    </div>
  );
}

type UseCaseCardProps = {
  title: string;
  subtitle: string;
  onClick: () => void;
};

function UseCaseCard(props: UseCaseCardProps) {
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
        className={`button ${
          readyToSubmit ? 'button--outline button--primary' : 'disabled button--secondary'
        }`}
        style={readyToSubmit ? {} : { color: '#f5f5f5', backgroundColor: '#dbdbdb' }}
        onClick={onSubmit}>
        <span style={{ fontSize: '1.2rem' }}>Submit</span>
      </button>
      <Tooltip id="disableSubmit" style={tooltipStyle} opacity={1} />
    </div>
  );
}

// function OutputResolution() {
//   return (
//     <div className={styles.outputResolutionsContainer}>
//       <div className={styles.outputResolutionLabel}>Output resolution:</div>

//       <select className={styles.outputResolutionSelect}>
//         <option value="Resoultion1920x1080">[16:9] 1920x1080</option>
//         <option value="Resoultion1080x1920">[9:16] 1080x1920</option>
//         <option value="Resoultion854x480">[16:9] 854x480</option>
//         <option value="Resoultion480x854">[9:16] 480x854</option>
//         <option value="Resoultion1440x1080">[4:3] 1440x1080</option>
//         <option value="Resoultion1080x1440">[3:4] 1080x1440</option>
//       </select>
//     </div>
//   );
// }
