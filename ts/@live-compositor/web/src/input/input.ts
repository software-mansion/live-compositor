import { Frame, InputId } from '@live-compositor/browser-render';
import MP4Source from './mp4/source';
import { CompositorEventType } from 'live-compositor';
import { EventSender } from '../eventSender';
import InputSource from './source';
import { RegisterInput } from './registerInput';

export type InputFrame = Frame & {
  free: () => void;
};

export type InputState = 'waiting_for_start' | 'buffering' | 'playing' | 'finished';

export class Input {
  private id: InputId;
  private source: InputSource;
  private state: InputState;
  private eventSender: EventSender;

  private constructor(id: InputId, source: InputSource, eventSender: EventSender) {
    this.id = id;
    this.source = source;
    this.state = 'waiting_for_start';
    this.eventSender = eventSender;
  }

  public static create(inputId: InputId, request: RegisterInput, eventSender: EventSender): Input {
    if (request.type === 'mp4') {
      return new Input(inputId, new MP4Source(request.url), eventSender);
    } else {
      throw new Error(`Unknown input type ${(request as any).type}`);
    }
  }

  public start() {
    if (this.state != 'waiting_for_start') {
      console.warn(`Tried to start an already started input "${this.id}"`);
      return;
    }
    this.source.start();
    this.state = 'buffering';
    this.eventSender.sendEvent({
      type: CompositorEventType.VIDEO_INPUT_DELIVERED,
      input_id: this.id,
    });
  }

  public async getFrame(): Promise<InputFrame | undefined> {
    const frame = await this.source.getFrame();
    // TODO(noituri): Handle this better
    if (frame && this.state == 'buffering') {
      this.state = 'playing';
      this.eventSender.sendEvent({
        type: CompositorEventType.VIDEO_INPUT_PLAYING,
        input_id: this.id,
      });
    }
    // TODO(noituri): Handle EOS

    return frame;
  }
}
