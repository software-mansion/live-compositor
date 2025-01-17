import type { Framerate } from '../compositor';

export type SourcePayload = { type: 'chunk'; chunk: EncodedVideoChunk } | { type: 'eos' };

export type SourceMetadata = {
  framerate?: Framerate;
  videoDurationMs?: number;
};

export type InputSourceCallbacks = {
  onDecoderConfig: (config: VideoDecoderConfig) => void;
};

/**
 * `InputSource` produces encoded video chunks required for decoding.
 */
export default interface InputSource {
  init(): Promise<void>;
  /**
   * Starts producing chunks. `init()` has to be called beforehand.
   */
  start(): Promise<void>;
  registerCallbacks(callbacks: InputSourceCallbacks): void;
  /**
   * if `true` InputSource won't produce more chunks anymore.
   */
  isFinished(): boolean;
  getMetadata(): SourceMetadata;
  nextChunk(): EncodedVideoChunk | undefined;
  peekChunk(): EncodedVideoChunk | undefined;
}
