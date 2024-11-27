import type { Framerate } from './compositor';

export function assert<T>(value?: T): T {
  if (!value) {
    throw new Error('Assertion failed');
  }

  return value;
}

export function framerateToDurationMs(framerate: Framerate): number {
  return (1000 * framerate.den) / framerate.num;
}
