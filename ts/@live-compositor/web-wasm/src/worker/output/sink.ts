import type { Frame } from '@live-compositor/browser-render';

export interface OutputSink {
  send(frame: Frame): Promise<void>;
}
