import { useState } from 'react';
import { Tooltip } from 'react-tooltip';
import { Resolution } from '../resolution';
import styles from './PlaygroundSettingsOutput.module.css';

interface OutputResolutionProps {
  resolution: Resolution;
  handleSettingsUpdate: (outputResolution: Resolution) => void;
  setValidity: (validity: boolean) => void;
}

export default function OutputResolution({
  resolution,
  handleSettingsUpdate,
  setValidity,
}: OutputResolutionProps) {
  return (
    <div className={styles.outputResolutionsContainer}>
      <label className={styles.outputResolutionLabel}>Output resolution:</label>
      <div className={styles.resolutionInputFieldsContainer}>
        <ResolutionInputField
          id="width"
          value={resolution.width}
          maxValue={7682}
          onValueChange={(width: number) => {
            const newResolution: Resolution = {
              width: width,
              height: resolution.height,
            };
            handleSettingsUpdate(newResolution);
          }}
          setValidity={setValidity}
        />
        <span style={{ margin: 2 }}>&#215;</span>
        <ResolutionInputField
          id="height"
          value={resolution.height}
          maxValue={4320}
          onValueChange={(height: number) => {
            const newResolution: Resolution = {
              width: resolution.width,
              height: height,
            };
            console.log(newResolution);
            handleSettingsUpdate(newResolution);
          }}
          setValidity={setValidity}
        />
      </div>
    </div>
  );
}

enum ValidationResult {
  Ok = 'Ok',
  TooLargeError = 'TooLarge',
  TooSmallError = 'TooSmall',
  UnevenError = 'Uneven',
  ParsingError = 'ParsingError',
}

interface ResolutionInputFieldProps {
  id: string;
  value: number;
  maxValue: number;
  onValueChange: (number) => void;
  setValidity: (boolean) => void;
}

function ResolutionInputField({
  id,
  value,
  maxValue,
  onValueChange,
  setValidity,
}: ResolutionInputFieldProps) {
  const [validationResult, setValidationResult] = useState<ValidationResult>(ValidationResult.Ok);
  function setInputValidationResult(result: ValidationResult) {
    if (result == ValidationResult.Ok) {
      setValidity(true);
    } else {
      setValidity(false);
    }
    setValidationResult(result);
  }

  function parseAndValidate(input_string: string): number {
    const input_val = parseInt(input_string);
    if (Number.isNaN(input_val)) {
      setInputValidationResult(ValidationResult.ParsingError);
    } else if (input_val % 2 == 1) {
      setInputValidationResult(ValidationResult.UnevenError);
    } else if (input_val > maxValue) {
      setInputValidationResult(ValidationResult.TooLargeError);
    } else if (input_val <= 0) {
      setInputValidationResult(ValidationResult.TooSmallError);
    } else {
      setInputValidationResult(ValidationResult.Ok);
    }
    return input_val;
  }

  function updateInputValue(event) {
    const value = parseAndValidate(event.target.value);
    console.log(value);
    if (!Number.isNaN(value)) {
      onValueChange(value);
    }
  }

  return (
    <div>
      <input
        id={id}
        className={`${styles.outputResolutionInput} ${
          validationResult == ValidationResult.Ok ? null : styles.invalidInput
        }`}
        type="number"
        value={value}
        onChange={updateInputValue}
      />
      <Tooltip
        anchorSelect={`#${id}`}
        className={`${styles.tooltip} ${
          validationResult == ValidationResult.Ok ? styles.inactiveTooltip : null
        }`}
        delayShow={128}>
        {validationResultMessage(validationResult, maxValue)}
      </Tooltip>
    </div>
  );
}

function validationResultMessage(validationResult: ValidationResult, maxValue: number) {
  if (validationResult == ValidationResult.Ok) {
    return 'Everything is fine';
  } else if (validationResult == ValidationResult.ParsingError) {
    return "Value isn't a valid number";
  } else if (validationResult == ValidationResult.TooLargeError) {
    return `Value has to be not greater than ${maxValue}`;
  } else if (validationResult == ValidationResult.TooSmallError) {
    return 'Value has to be greater than 0';
  } else if (validationResult == ValidationResult.UnevenError) {
    return 'Value has to be even';
  }
}
