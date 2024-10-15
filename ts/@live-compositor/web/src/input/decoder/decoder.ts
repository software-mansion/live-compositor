import { VideoPayload } from '../payload';

export type DecoderCallbacks = {
  onPayload: (payload: VideoPayload) => void;
};

export default interface Decoder {
  configure(config: VideoDecoderConfig): void;
  registerCallbacks(callbacks: DecoderCallbacks): void;
  enqueue(chunk: EncodedVideoChunk): void;
  isClosed(): boolean;
  close(): Promise<void>;
  decodeQueueSize(): number;
}
