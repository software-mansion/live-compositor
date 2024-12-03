import type { Framerate } from './compositor';

export function assert<T>(value: T): asserts value {
  if (!value) {
    throw new Error('Assertion failed');
  }
}

export function framerateToDurationMs(framerate: Framerate): number {
  return (1000 * framerate.den) / framerate.num;
}
