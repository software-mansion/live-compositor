import { RegisterInputRequest } from "@live-compositor/core";
import { FrameRef } from "./frame";
import DecodingFrameProducer from "./producer/decodingFrameProducer";
import MP4Source from "./mp4/source";
import CameraFrameProducer from "./producer/cameraFrameProducer";

export const DEFAULT_MAX_BUFFERING_SIZE = 3;

export type InputFrameProducerCallbacks = {
  onFrame(frame: FrameRef): void;
};

// TODO(noituri): Comment it
export default interface InputFrameProducer {
  init(): Promise<void>;
  start(): void;
  registerCallbacks(callbacks: InputFrameProducerCallbacks): void;
  produce(framePts?: number): Promise<void>;
  setMaxBufferSize(maxBufferSize: number): void;
  isFinished(): boolean;
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
