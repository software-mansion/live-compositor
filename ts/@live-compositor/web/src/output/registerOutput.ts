import { Resolution } from '@live-compositor/browser-render';
import { RegisterOutput as InternalRegisterOutput, OutputByteFormat } from 'live-compositor';

export type RegisterOutput = { type: 'canvas' } & RegisterCanvasOutput;

export type RegisterCanvasOutput = {
  resolution: Resolution;
  canvas: HTMLCanvasElement;
  root: React.ReactElement;
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
    type: 'bytes',
    video: {
      resolution: output.resolution,
      format: OutputByteFormat.RGBA_BYTES,
      root: output.root,
    },
  };
}
