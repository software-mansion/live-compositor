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
  const [resolutionInputsValidity, setResolutionInputsValidity] = useState({
    width: true,
    height: true,
  });
  return (
    <div className={styles.outputResolutionsContainer}>
      <label className={styles.outputResolutionLabel}>Output resolution:</label>
      <div className={styles.resolutionInputFieldsContainer}>
        <ResolutionInputField
          id="width"
          value={resolution.width}
          maxValue={7682}
          onValueChange={(width: number) => {
            handleSettingsUpdate({
              width: width,
              height: resolution.height,
            });
          }}
          setValidity={(widthValidity: boolean) => {
            setResolutionInputsValidity({
              width: widthValidity,
              height: resolutionInputsValidity.height,
            });
            setValidity(widthValidity && resolutionInputsValidity.height);
          }}
        />
        <span style={{ margin: 2 }}>&#215;</span>
        <ResolutionInputField
          id="height"
          value={resolution.height}
          maxValue={4320}
          onValueChange={(height: number) => {
            handleSettingsUpdate({
              width: resolution.width,
              height: height,
            });
          }}
          setValidity={(heightValidity: boolean) => {
            setResolutionInputsValidity({
              width: resolutionInputsValidity.width,
              height: heightValidity,
            });
            setValidity(resolutionInputsValidity.width && heightValidity);
          }}
        />
      </div>
    </div>
  );
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

  function updateInputValue(event) {
    const value = parseAndValidate(event.target.value, maxValue, setInputValidationResult);
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

function parseAndValidate(
  input_string: string,
  maxValue: number,
  setInputValidationResult: (ValidationResult) => void
): number {
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

enum ValidationResult {
  Ok = 'Ok',
  TooLargeError = 'TooLarge',
  TooSmallError = 'TooSmall',
  UnevenError = 'Uneven',
  ParsingError = 'ParsingError',
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
