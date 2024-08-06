export type InputResolutions = {
  input_1: Resolution;
  input_2: Resolution;
  input_3: Resolution;
  input_4: Resolution;
  input_5: Resolution;
  input_6: Resolution;
};

export type Resolution = {
  width: number;
  height: number;
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
