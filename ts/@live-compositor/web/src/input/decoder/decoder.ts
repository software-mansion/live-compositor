import { InputFrame } from '../input';
import { VideoPayload } from '../payload';

export default interface Decoder {
  configure(config: VideoDecoderConfig): void;
  enqueue(payload: VideoPayload): void;
  getFrame(): Promise<InputFrame | undefined>;
  isFinished(): boolean;
  isBufferFull(): boolean;
}
