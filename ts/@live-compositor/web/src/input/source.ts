import { VideoPayload } from './payload';

export type SourceCallbacks = {
  onDecoderConfig: (config: VideoDecoderConfig) => void;
  onPayload: (payload: VideoPayload) => void;
};

/**
 * Represents `EncodedVideoChunk` producer
 */
export default interface InputSource {
  start(): Promise<void>;
  registerCallbacks(callbacks: SourceCallbacks): void;
}
