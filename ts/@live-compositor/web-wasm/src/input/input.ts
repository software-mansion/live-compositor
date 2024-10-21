import { Frame, InputId } from '@live-compositor/browser-render';
import { CompositorEventType } from 'live-compositor';
import { EventSender } from '../eventSender';
import InputSource from './source';

/**
 * Represents frame produced by decoder.
 * `InputFrame` has to be manually freed from the memory by calling `free()` method. Once freed it no longer can be used.
 * `Queue` on tick pulls `InputFrame` for each input and once render finishes, manually frees `InputFrame`s.
 */
export type InputFrame = Frame & {
  /**
   * Frees `InputFrame` from memory. `InputFrame` can not be used after `free()`.
   */
  free: () => void;
};

export type InputState = 'waiting_for_start' | 'buffering' | 'playing' | 'finished';

export class Input {
  private id: InputId;
  private source: InputSource;
  private state: InputState;
  private eventSender: EventSender;

  public constructor(id: InputId, source: InputSource, eventSender: EventSender) {
    this.id = id;
    this.state = 'waiting_for_start';
    this.source = source;
    this.eventSender = eventSender;
  }

  public start() {
    if (this.state !== 'waiting_for_start') {
      console.warn(`Tried to start an already started input "${this.id}"`);
      return;
    }
    this.source.start();
    this.state = 'buffering';
    this.eventSender.sendEvent({
      type: CompositorEventType.VIDEO_INPUT_DELIVERED,
      inputId: this.id,
    });
  }

  public async getFrame(): Promise<InputFrame | undefined> {
    const frame = await this.source.getFrame();
    // TODO(noituri): Handle this better
    if (frame && this.state === 'buffering') {
      this.state = 'playing';
      this.eventSender.sendEvent({
        type: CompositorEventType.VIDEO_INPUT_PLAYING,
        inputId: this.id,
      });
    }
    // TODO(noituri): Handle EOS

    return frame;
  }
}
