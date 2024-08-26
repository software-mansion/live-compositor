import { useState } from 'react';
import { Resolution } from '../resolution';
import styles from './PlaygroundSettingsOutput.module.css';

interface OutputResolutionProps {
  handleSettingsUpdate: (outputResolution: Resolution) => void;
  setValidity: (validity: boolean) => void;
}

export default function OutputResolution({
  handleSettingsUpdate,
  setValidity,
}: OutputResolutionProps) {
  const [resolution, setResolution] = useState<Resolution>({ width: 1920, height: 1080 });
  const [isWidthValid, setIsWidthValid] = useState<boolean>(true);
  const [isHeightValid, setIsHeightValid] = useState<boolean>(true);

  function setWidthValidity(validity: boolean) {
    setValidity(validity);
    setIsWidthValid(validity);
  }
  function setHeightValidity(validity: boolean) {
    setValidity(validity);
    setIsHeightValid(validity);
  }

  function parseAndValidateHeight(height_string: string) {
    const height = parseInt(height_string);
    if (Number.isNaN(height) || height % 2 == 1 || height > 4320) {
      setHeightValidity(false);
    } else {
      setHeightValidity(true);
      return height;
    }
  }

  function parseAndValidateWidth(width_string: string) {
    const width = parseInt(width_string);
    if (Number.isNaN(width) || width % 2 == 1 || width > 7682) {
      setWidthValidity(false);
    } else {
      setWidthValidity(true);
      return width;
    }
  }

  function updateWidth(event) {
    const newResolution: Resolution = {
      width: parseAndValidateWidth(event.target.value),
      height: resolution.height,
    };
    handleSettingsUpdate(newResolution);
    setResolution(newResolution);
  }

  function updateHeight(event) {
    const newResolution: Resolution = {
      width: resolution.width,
      height: parseAndValidateHeight(event.target.value),
    };
    handleSettingsUpdate(newResolution);
    setResolution(newResolution);
  }

  return (
    <div className={styles.outputResolutionsContainer}>
      <div className={styles.outputResolutionLabel}>Output resolution:</div>
      <div>
        <input
          id="width"
          className={`${styles.outputResolutionInput} ${isWidthValid ? null : styles.invalidInput}`}
          type="number"
          value={resolution.width}
          onChange={updateWidth}
        />
        <span style={{ margin: 2 }}>&#215;</span>
        <input
          id="height"
          className={`${styles.outputResolutionInput} ${
            isHeightValid ? null : styles.invalidInput
          }`}
          type="number"
          value={resolution.height}
          onChange={updateHeight}
        />
      </div>
    </div>
  );
}
