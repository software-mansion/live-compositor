import { InputFrame } from './input';

export default interface InputSource {
  start(): Promise<void>;
  getFrame(): Promise<InputFrame | undefined>;
}
