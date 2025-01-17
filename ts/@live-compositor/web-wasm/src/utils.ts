import type { Framerate } from './compositor';

export function assert<T>(value: T, msg?: string): asserts value {
  if (!value) {
    if (msg) {
      throw new Error(msg);
    } else {
      throw new Error('Assertion failed');
    }
  }
}

export function framerateToDurationMs(framerate: Framerate): number {
  return (1000 * framerate.den) / framerate.num;
}
