import { RegisterInputRequest } from "@live-compositor/core";
import { FrameRef } from "./frame";
import DecodingFrameProducer from "./producer/decodingFrameProducer";
import MP4Source from "./mp4/source";

export const DEFAULT_MAX_BUFFERING_SIZE = 3;

// TODO(noituri): Comment it
export default interface InputFrameProducer {
  init(): Promise<void>;
  start(): void;
  produce(framePts: number): Promise<void>;
  nextFrame(): FrameRef | undefined;
  peekFrame(): FrameRef | undefined;
  frameCount(): number;
  setMaxBufferSize(maxBufferSize: number): void;
  isFinished(): boolean;
}

export function producerFromRequest(request: RegisterInputRequest): InputFrameProducer {
  if (request.type === 'mp4') {
    return new DecodingFrameProducer(new MP4Source(request.url!));
  } else {
    throw new Error(`Unknown input type ${(request as any).type}`);
  }
}
