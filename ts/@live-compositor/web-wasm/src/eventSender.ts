import type { CompositorEvent } from 'live-compositor';
import { CompositorEventType } from 'live-compositor';

export class EventSender {
  private eventCallback?: (event: object) => void;

  public setEventCallback(eventCallback: (event: object) => void) {
    this.eventCallback = eventCallback;
  }

  public sendEvent(event: CompositorEvent) {
    if (!this.eventCallback) {
      console.warn(`Failed to send event: ${event}`);
      return;
    }

    this.eventCallback!(toWebSocketMessage(event));
  }
}

function toWebSocketMessage(event: CompositorEvent): WebSocketMessage {
  if (event.type == CompositorEventType.OUTPUT_DONE) {
    return {
      type: event.type,
      output_id: event.outputId,
    };
  }

  return {
    type: event.type,
    input_id: event.inputId,
  };
}

export type WebSocketMessage =
  | {
      type: CompositorEventType.AUDIO_INPUT_DELIVERED;
      input_id: string;
    }
  | {
      type: CompositorEventType.VIDEO_INPUT_DELIVERED;
      input_id: string;
    }
  | {
      type: CompositorEventType.AUDIO_INPUT_PLAYING;
      input_id: string;
    }
  | {
      type: CompositorEventType.VIDEO_INPUT_PLAYING;
      input_id: string;
    }
  | {
      type: CompositorEventType.AUDIO_INPUT_EOS;
      input_id: string;
    }
  | {
      type: CompositorEventType.VIDEO_INPUT_EOS;
      input_id: string;
    }
  | {
      type: CompositorEventType.OUTPUT_DONE;
      output_id: string;
    };
