import type { Resolution } from '@live-compositor/browser-render';
import type { RegisterOutput as InternalRegisterOutput } from '@live-compositor/core';

export type RegisterOutput = { type: 'canvas' } & RegisterCanvasOutput;

export type RegisterCanvasOutput = {
  resolution: Resolution;
  canvas: HTMLCanvasElement;
};

export function intoRegisterOutput(output: RegisterOutput): InternalRegisterOutput {
  if (output.type === 'canvas') {
    return fromRegisterCanvasOutput(output);
  } else {
    throw new Error(`Unknown output type ${(output as any).type}`);
  }
}

function fromRegisterCanvasOutput(output: RegisterCanvasOutput): InternalRegisterOutput {
  return {
    type: 'canvas',
    video: {
      resolution: output.resolution,
      canvas: output.canvas,
    },
  };
}
