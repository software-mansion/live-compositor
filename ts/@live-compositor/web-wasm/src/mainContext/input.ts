import type { Input as CoreInput } from '@live-compositor/core';
import type { WorkerMessage } from '../workerApi';
import { assert } from '../utils';
import { handleRegisterCameraInput } from './input/camera';
import { handleRegisterScreenCaptureInput } from './input/screenCapture';

export interface Input {
  terminate(): Promise<void>;
}

class NoopInput implements Input {
  public async terminate(): Promise<void> {}
}

export type RegisterInputResult = {
  input: Input;
  workerMessage: [WorkerMessage, Transferable[]];
};

export async function handleRegisterInputRequest(
  inputId: string,
  body: CoreInput.RegisterInputRequest
): Promise<RegisterInputResult> {
  if (body.type === 'mp4') {
    assert(body.url, 'mp4 URL is required');
    return {
      input: new NoopInput(),
      workerMessage: [
        {
          type: 'registerInput',
          inputId,
          input: {
            type: 'mp4',
            url: body.url,
          },
        },
        [],
      ],
    };
  } else if (body.type === 'camera') {
    return await handleRegisterCameraInput(inputId);
  } else if (body.type === 'screen_capture') {
    return await handleRegisterScreenCaptureInput(inputId);
  } else {
    throw new Error(`Unknown input type ${body.type}`);
  }
}
