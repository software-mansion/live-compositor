import { _liveCompositorInternals } from 'live-compositor';

export const CompositorEventType = _liveCompositorInternals.CompositorEventType;
export const inputRefIntoRawId = _liveCompositorInternals.inputRefIntoRawId;

export class EventSender {
  private eventCallback?: (event: object) => void;

  public setEventCallback(eventCallback: (event: object) => void) {
    this.eventCallback = eventCallback;
  }

  public sendEvent(event: WasmCompositorEvent) {
    if (!this.eventCallback) {
      console.warn(`Failed to send event: ${event}`);
      return;
    }

    this.eventCallback!(toWebSocketMessage(event));
  }
}

function toWebSocketMessage(event: WasmCompositorEvent): WebSocketMessage {
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

export type WasmCompositorEvent =
  | {
      type:
        | _liveCompositorInternals.CompositorEventType.AUDIO_INPUT_DELIVERED
        | _liveCompositorInternals.CompositorEventType.VIDEO_INPUT_DELIVERED
        | _liveCompositorInternals.CompositorEventType.AUDIO_INPUT_PLAYING
        | _liveCompositorInternals.CompositorEventType.VIDEO_INPUT_PLAYING
        | _liveCompositorInternals.CompositorEventType.AUDIO_INPUT_EOS
        | _liveCompositorInternals.CompositorEventType.VIDEO_INPUT_EOS;
      inputId: string;
    }
  | {
      type: _liveCompositorInternals.CompositorEventType.OUTPUT_DONE;
      outputId: string;
    };
export type WebSocketMessage =
  | {
      type:
        | _liveCompositorInternals.CompositorEventType.AUDIO_INPUT_DELIVERED
        | _liveCompositorInternals.CompositorEventType.VIDEO_INPUT_DELIVERED
        | _liveCompositorInternals.CompositorEventType.AUDIO_INPUT_PLAYING
        | _liveCompositorInternals.CompositorEventType.VIDEO_INPUT_PLAYING
        | _liveCompositorInternals.CompositorEventType.AUDIO_INPUT_EOS
        | _liveCompositorInternals.CompositorEventType.VIDEO_INPUT_EOS;
      input_id: string;
    }
  | {
      type: _liveCompositorInternals.CompositorEventType.OUTPUT_DONE;
      output_id: string;
    };
