import { InputFrame } from './input';

export type VideoPayload =
  | { type: 'chunk'; data: EncodedVideoChunk }
  | { type: 'frame'; data: InputFrame }
  | { type: 'eos' };

export type OnPayload = (payload: VideoPayload) => void;
