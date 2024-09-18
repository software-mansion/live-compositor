import { InputFrame } from './input';

export default interface InputSource {
  start(): void;
  getFrame(): Promise<InputFrame | undefined>;
}
