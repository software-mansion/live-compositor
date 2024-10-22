import { RegisterInputRequest } from '@live-compositor/core';
import { InputFrame } from './input';
import MP4Source from './mp4/source';

export default interface InputSource {
  init(): Promise<void>;
  /**
   * Starts input processing. `init()` has to be called beforehand.
   */
  start(): void;
  getFrame(): Promise<InputFrame | undefined>;
}

export function sourceFromRequest(request: RegisterInputRequest): InputSource {
  if (request.type === 'mp4') {
    return new MP4Source(request.url!);
  } else {
    throw new Error(`Unknown input type ${(request as any).type}`);
  }
}
