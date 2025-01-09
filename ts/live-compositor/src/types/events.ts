import type { InputRef } from './refs/inputRef.js';

export enum CompositorEventType {
  AUDIO_INPUT_DELIVERED = 'AUDIO_INPUT_DELIVERED',
  VIDEO_INPUT_DELIVERED = 'VIDEO_INPUT_DELIVERED',
  AUDIO_INPUT_PLAYING = 'AUDIO_INPUT_PLAYING',
  VIDEO_INPUT_PLAYING = 'VIDEO_INPUT_PLAYING',
  AUDIO_INPUT_EOS = 'AUDIO_INPUT_EOS',
  VIDEO_INPUT_EOS = 'VIDEO_INPUT_EOS',
  OUTPUT_DONE = 'OUTPUT_DONE',
}

export type CompositorEvent =
  | { type: CompositorEventType.AUDIO_INPUT_DELIVERED; inputRef: InputRef }
  | { type: CompositorEventType.VIDEO_INPUT_DELIVERED; inputRef: InputRef }
  | { type: CompositorEventType.AUDIO_INPUT_PLAYING; inputRef: InputRef }
  | { type: CompositorEventType.VIDEO_INPUT_PLAYING; inputRef: InputRef }
  | { type: CompositorEventType.AUDIO_INPUT_EOS; inputRef: InputRef }
  | { type: CompositorEventType.VIDEO_INPUT_EOS; inputRef: InputRef }
  | { type: CompositorEventType.OUTPUT_DONE; outputId: string };
