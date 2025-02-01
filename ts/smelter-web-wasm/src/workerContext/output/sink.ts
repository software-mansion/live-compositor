import type { Frame } from '@swmansion/smelter-browser-render';

export interface OutputSink {
  send(frame: Frame): Promise<void>;
}
