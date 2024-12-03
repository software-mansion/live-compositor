import type { Framerate } from '../compositor';

export type SourcePayload = { type: 'chunk'; chunk: EncodedVideoChunk } | { type: 'eos' };

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
  start(): void;
  registerCallbacks(callbacks: InputSourceCallbacks): void;
  /**
   * if `true` InputSource won't produce more chunks anymore.
   */
  isFinished(): boolean;
  getFramerate(): Framerate | undefined;
  nextChunk(): EncodedVideoChunk | undefined;
  peekChunk(): EncodedVideoChunk | undefined;
}
