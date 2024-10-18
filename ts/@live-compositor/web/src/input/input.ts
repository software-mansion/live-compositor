import { Frame, InputId } from '@live-compositor/browser-render';
import MP4Source from './mp4/source';
import { CompositorEventType } from 'live-compositor';
import { EventSender } from '../eventSender';
import InputSource from './source';
import { RegisterInputRequest } from '@live-compositor/core';

/**
 * Represents frame produced by decoder. All `InputFrame`s have to be manually freed.
 */
export type InputFrame = Frame & {
  /**
   * Frees InputFrame from memory. InputFrame can not be used after `free()`.
   */
  free: () => void;
};

export type InputState = 'waiting_for_start' | 'buffering' | 'playing' | 'finished';

export class Input {
  private id: InputId;
  private source: InputSource;
  private state: InputState;
  private eventSender: EventSender;

  public constructor(id: InputId, request: RegisterInputRequest, eventSender: EventSender) {
    this.id = id;
    this.state = 'waiting_for_start';
    this.eventSender = eventSender;
    if (request.type === 'mp4') {
      this.source = new MP4Source(request.url!);
    } else {
      throw new Error(`Unknown input type ${(request as any).type}`);
    }
  }

  public async start() {
    if (this.state !== 'waiting_for_start') {
      console.warn(`Tried to start an already started input "${this.id}"`);
      return;
    }
    await this.source.start();
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
