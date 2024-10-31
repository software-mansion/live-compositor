import type { RegisterInput as InternalRegisterInput } from '@live-compositor/core';

export type RegisterInput = { type: 'mp4' } & RegisterMP4Input;

export type RegisterMP4Input = {
  url: string;
};

export function intoRegisterInput(input: RegisterInput): InternalRegisterInput {
  if (input.type === 'mp4') {
    return { type: 'mp4', url: input.url };
  } else {
    throw new Error(`Unknown input type ${(input as any).type}`);
  }
}
