import type { RegisterInput as CoreRegisterInput } from '@live-compositor/core';

export type RegisterInput =
  | ({ type: 'mp4' } & RegisterMP4Input)
  | { type: 'camera' }
  | { type: 'screen_capture' };

export type RegisterMP4Input = {
  url: string;
};

export function intoRegisterInput(input: RegisterInput): CoreRegisterInput {
  if (input.type === 'mp4') {
    return { type: 'mp4', url: input.url };
  } else if (input.type === 'camera') {
    return { type: 'camera' };
  } else if (input.type === 'screen_capture') {
    return { type: 'screen_capture' };
  } else {
    throw new Error(`Unknown input type ${(input as any).type}`);
  }
}
