import { InputResolutionNames, ResolutionName } from '../resolution';
import styles from './PlaygroundSettingsInputs.module.css';

interface PlaygroundSettingsInputsProps {
  handleSettingsUpdate: (input_id: string, resolution: ResolutionName) => void;
  inputResolutions: InputResolutionNames;
}

export default function PlaygroundSettingsInputs({
  handleSettingsUpdate,
  inputResolutions,
}: PlaygroundSettingsInputsProps) {
  function handleChange(event, inputId: string) {
    handleSettingsUpdate(inputId, event.target.value);
  }

  return (
    <div className={styles.container}>
      {/* <div className={`${styles.headerContainer}`}> */}
      <div className={`${styles.headerContainer} row`}>
        {/* <div className={`${styles.headerLabel}`}>Input ID</div> */}
        <div className={`${styles.headerLabel} col col--3`}>Input ID</div>
        {/* <div className={`${styles.headerLabel}`}>Resolution</div> */}
        <div className={`${styles.headerLabel} col col--5`}>Resolution</div>
        {/* <div className={`${styles.headerLabel}`}>Preview</div> */}
        <div className={`${styles.headerLabel} col col--4`}>Preview</div>
      </div>
      {Object.keys(inputResolutions).map(inputId => (
        <InputResolutionSelect
          inputName={inputId}
          selectedValue={inputResolutions[inputId]}
          handleChange={event => handleChange(event, inputId)}
          key={inputId}
        />
      ))}
    </div>
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
    // <div className={`${styles.inputSelector}`}>
    <div className={`${styles.inputSelector} row`}>
      {/* <label className={`${styles.inputSelectorLabel}`}>{inputName}</label> */}
      <label className={`${styles.inputSelectorLabel} col col--3`}>{inputName}</label>
      {/* <div className={`${styles.inputSelectorSelectContainer}`}> */}
      <div className={`${styles.inputSelectorSelectContainer} col col--5`}>
        <select
          onChange={handleChange}
          value={selectedValue}
          className={styles.inputSelectorSelect}>
          <option value={ResolutionName.Resoultion1920x1080}>[16:9] 1920x1080</option>
          <option value={ResolutionName.Resoultion1080x1920}>[9:16] 1080x1920</option>
          <option value={ResolutionName.Resoultion854x480}>[16:9] 854x480</option>
          <option value={ResolutionName.Resoultion480x854}>[9:16] 480x854</option>
          <option value={ResolutionName.Resoultion1440x1080}>[4:3] 1440x1080</option>
          <option value={ResolutionName.Resoultion1080x1440}>[3:4] 1080x1440</option>
        </select>
      </div>
      {/* <div className={styles.inputSelectorImgContainer}> */}
      <img
        src={getImagePath(inputName, selectedValue)}
        alt={'alt'}
        //   className={`${styles.inputSelectorImg}`}
        className={`${styles.inputSelectorImg} col col--4`}
      />
    </div>
    // </div>
  );
}

function getImagePath(inputName: string, resolutionName: ResolutionName): string {
  return `/img/inputs/${inputName}_${nameToMiniature(resolutionName)}.webp`;
}

function nameToMiniature(resolutionName: ResolutionName): string {
  switch (resolutionName) {
    case ResolutionName.Resoultion1920x1080:
    case ResolutionName.Resoultion854x480:
      return '640x360';
    case ResolutionName.Resoultion1080x1920:
    case ResolutionName.Resoultion480x854:
      return '360x640';
    case ResolutionName.Resoultion1440x1080:
      return '640x480';
    case ResolutionName.Resoultion1080x1440:
      return '480x640';
  }
}
