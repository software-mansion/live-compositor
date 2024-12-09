import { RegisterInputRequest } from "@live-compositor/core";
import { FrameRef } from "./frame";
import DecodingFrameProducer from "./producer/decodingFrameProducer";
import MP4Source from "./mp4/source";
import CameraFrameProducer from "./producer/cameraFrameProducer";

export type InputFrameProducerCallbacks = {
  onFrame(frame: FrameRef): void;
};

export default interface InputFrameProducer {
  init(): Promise<void>;
  /**
   * Starts resources required for producing frames. `init()` has to be called beforehand.
   */
  start(): void;
  registerCallbacks(callbacks: InputFrameProducerCallbacks): void;
  /**
   * Produce next frame.
   * @param framePts - Desired PTS of the frame in milliseconds.
   */
  produce(framePts?: number): Promise<void>;
  /**
   * if `true` no more frames will be produced.
   */
  isFinished(): boolean;
  maxBufferSize(): number;
  close(): void;
}

export function producerFromRequest(request: RegisterInputRequest): InputFrameProducer {
  if (request.type === 'mp4') {
    return new DecodingFrameProducer(new MP4Source(request.url!));
  } else if (request.type === 'camera') {
    return new CameraFrameProducer();
  } else {
    throw new Error(`Unknown input type ${(request as any).type}`);
  }
}
