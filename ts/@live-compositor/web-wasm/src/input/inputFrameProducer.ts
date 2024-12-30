import type { RegisterInputRequest } from '@live-compositor/core';
import type { FrameRef } from './frame';
import DecodingFrameProducer from './producer/decodingFrameProducer';
import MediaStreamFrameProducer from './producer/mediaStreamFrameProducer';
import MP4Source from './mp4/source';
import { initCameraMediaStream } from './producer/mediaStreamInit';

export type InputFrameProducerCallbacks = {
  onReady(): void;
};

export default interface InputFrameProducer {
  init(): Promise<void>;
  /**
   * Starts resources required for producing frames. `init()` has to be called beforehand.
   */
  start(): Promise<void>;
  registerCallbacks(callbacks: InputFrameProducerCallbacks): void;
  /**
   * Produce next frame.
   * @param framePts - Desired PTS of the frame in milliseconds.
   */
  produce(framePts?: number): Promise<void>;
  getFrameRef(framePts: number): FrameRef | undefined;
  /**
   * if `true` no more frames will be produced.
   */
  isFinished(): boolean;
  close(): void;
}

export function producerFromRequest(request: RegisterInputRequest): InputFrameProducer {
  if (request.type === 'mp4') {
    return new DecodingFrameProducer(new MP4Source(request.url!));
  } else if (request.type === 'camera') {
    return new MediaStreamFrameProducer(initCameraMediaStream);
  } else {
    throw new Error(`Unknown input type ${(request as any).type}`);
  }
}
