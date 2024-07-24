export type Input = {
  inputId: string;
  inputType: 'rtp' | 'mp4';
};

export type Context = {
  inputs: Input[];
};
