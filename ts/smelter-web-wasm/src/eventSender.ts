import { _smelterInternals } from '@swmansion/smelter';
import type { WorkerEvent } from './workerApi';

export const SmelterEventType = _smelterInternals.SmelterEventType;
export const inputRefIntoRawId = _smelterInternals.inputRefIntoRawId;

export class EventSender {
  private eventCallbacks: Set<(event: object) => void> = new Set();

  /**
   * Check if this is event that should be passed to core
   */
  public static isExternalEvent(event: WorkerEvent): event is ExternalWorkerEvent {
    return Object.values(SmelterEventType).includes(event?.type);
  }

  public registerEventCallback(eventCallback: (event: object) => void) {
    this.eventCallbacks?.add(eventCallback);
  }

  public sendEvent(event: ExternalWorkerEvent) {
    for (const cb of this.eventCallbacks) {
      cb(toWebSocketMessage(event));
    }
  }
}

function toWebSocketMessage(event: ExternalWorkerEvent): WebSocketMessage {
  if (event.type == SmelterEventType.OUTPUT_DONE) {
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

/**
 * Subset of WorkerEvents that should be passed outside (to the core code)
 */
export type ExternalWorkerEvent =
  | {
      type:
        | _smelterInternals.SmelterEventType.AUDIO_INPUT_DELIVERED
        | _smelterInternals.SmelterEventType.VIDEO_INPUT_DELIVERED
        | _smelterInternals.SmelterEventType.AUDIO_INPUT_PLAYING
        | _smelterInternals.SmelterEventType.VIDEO_INPUT_PLAYING
        | _smelterInternals.SmelterEventType.AUDIO_INPUT_EOS
        | _smelterInternals.SmelterEventType.VIDEO_INPUT_EOS;
      inputId: string;
    }
  | {
      type: _smelterInternals.SmelterEventType.OUTPUT_DONE;
      outputId: string;
    };

/**
 * Actual format that in non-WASM smelter would be sent via WebSocket. Here it's only used to match the format
 * so the core package can handle both WASM and non-WASM instances.
 */
export type WebSocketMessage =
  | {
      type:
        | _smelterInternals.SmelterEventType.AUDIO_INPUT_DELIVERED
        | _smelterInternals.SmelterEventType.VIDEO_INPUT_DELIVERED
        | _smelterInternals.SmelterEventType.AUDIO_INPUT_PLAYING
        | _smelterInternals.SmelterEventType.VIDEO_INPUT_PLAYING
        | _smelterInternals.SmelterEventType.AUDIO_INPUT_EOS
        | _smelterInternals.SmelterEventType.VIDEO_INPUT_EOS;
      input_id: string;
    }
  | {
      type: _smelterInternals.SmelterEventType.OUTPUT_DONE;
      output_id: string;
    };
