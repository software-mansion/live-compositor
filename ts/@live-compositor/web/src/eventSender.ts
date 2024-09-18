import { InputId } from '@live-compositor/browser-render';
import { CompositorEventType } from 'live-compositor';

export class EventSender {
  private eventCallback: (event: object) => void;

  public constructor(eventCallback: (event: object) => void) {
    this.eventCallback = eventCallback;
  }

  public sendEvent(event: ApiEvent) {
    this.eventCallback(event);
  }
}

export type ApiEvent =
  | {
      type: CompositorEventType.VIDEO_INPUT_DELIVERED;
      input_id: InputId;
    }
  | {
      type: CompositorEventType.VIDEO_INPUT_PLAYING;
      input_id: InputId;
    };
