import type { Resolution } from '@live-compositor/browser-render';
import type { RegisterOutput as CoreRegisterOutput } from '@live-compositor/core';

export type RegisterOutput = {
  type: 'canvas';
  video: {
    canvas: HTMLCanvasElement;
    resolution: Resolution;
  };
};

export function intoRegisterOutput(output: RegisterOutput): CoreRegisterOutput {
  if (output.type === 'canvas') {
    return {
      type: 'canvas',
      video: {
        resolution: output.video.resolution,
        canvas: output.video.canvas,
      },
    };
  } else {
    throw new Error(`Unknown output type ${(output as any).type}`);
  }
}
