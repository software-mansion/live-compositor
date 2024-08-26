export type Resolution = {
  width: number;
  height: number;
};

export type InputResolutions = {
  input_1: Resolution;
  input_2: Resolution;
  input_3: Resolution;
  input_4: Resolution;
  input_5: Resolution;
  input_6: Resolution;
};

export const AVAILABLE_RESOLUTIONS = {
  Resoultion1080x1920: {
    width: 1080,
    height: 1920,
  },
  Resoultion1920x1080: {
    width: 1920,
    height: 1080,
  },
  Resoultion854x480: {
    width: 854,
    height: 480,
  },
  Resoultion480x854: {
    width: 480,
    height: 854,
  },
  Resoultion1440x1080: {
    width: 1440,
    height: 1080,
  },
  Resoultion1080x1440: {
    width: 1080,
    height: 1440,
  },
} as const;

export enum InputResolution {
  Resoultion1080x1920 = 'Resoultion1080x1920',
  Resoultion1920x1080 = 'Resoultion1920x1080',
  Resoultion854x480 = 'Resoultion854x480',
  Resoultion480x854 = 'Resoultion480x854',
  Resoultion1440x1080 = 'Resoultion1440x1080',
  Resoultion1080x1440 = 'Resoultion1080x1440',
}

export type InputsSettings = {
  input_1: InputResolution;
  input_2: InputResolution;
  input_3: InputResolution;
  input_4: InputResolution;
  input_5: InputResolution;
  input_6: InputResolution;
};

export function inputResolutionsToResolutions(
  inputResolutionNames: InputsSettings
): InputResolutions {
  return {
    input_1: inputResolutionToResolution(inputResolutionNames.input_1),
    input_2: inputResolutionToResolution(inputResolutionNames.input_2),
    input_3: inputResolutionToResolution(inputResolutionNames.input_3),
    input_4: inputResolutionToResolution(inputResolutionNames.input_4),
    input_5: inputResolutionToResolution(inputResolutionNames.input_5),
    input_6: inputResolutionToResolution(inputResolutionNames.input_6),
  };
}

function inputResolutionToResolution(inputResolution: InputResolution): Resolution {
  return AVAILABLE_RESOLUTIONS[inputResolution];
}
