import { _liveCompositorInternals, CompositorEvent, CompositorEventType } from 'live-compositor';

type InstanceContextStore = _liveCompositorInternals.InstanceContextStore;

export function onCompositorEvent(store: InstanceContextStore, rawEvent: unknown) {
  const event = parseEvent(rawEvent);
  if (!event) {
    return;
  } else if (event.type === CompositorEventType.VIDEO_INPUT_DELIVERED) {
    store.dispatchUpdate({
      type: 'update_input',
      input: { inputId: event.inputId, videoState: 'ready' },
    });
  } else if (event.type === CompositorEventType.VIDEO_INPUT_PLAYING) {
    store.dispatchUpdate({
      type: 'update_input',
      input: { inputId: event.inputId, videoState: 'playing' },
    });
  } else if (event.type === CompositorEventType.VIDEO_INPUT_EOS) {
    store.dispatchUpdate({
      type: 'update_input',
      input: { inputId: event.inputId, videoState: 'finished' },
    });
  } else if (event.type === CompositorEventType.AUDIO_INPUT_DELIVERED) {
    store.dispatchUpdate({
      type: 'update_input',
      input: { inputId: event.inputId, audioState: 'ready' },
    });
  } else if (event.type === CompositorEventType.AUDIO_INPUT_PLAYING) {
    store.dispatchUpdate({
      type: 'update_input',
      input: { inputId: event.inputId, audioState: 'playing' },
    });
  } else if (event.type === CompositorEventType.AUDIO_INPUT_EOS) {
    store.dispatchUpdate({
      type: 'update_input',
      input: { inputId: event.inputId, audioState: 'finished' },
    });
  }
}

function parseEvent(event: any): CompositorEvent | null {
  if (!event.type) {
    console.error(`Malformed event: ${event}`);
    return null;
  } else if (
    [
      CompositorEventType.VIDEO_INPUT_DELIVERED,
      CompositorEventType.AUDIO_INPUT_DELIVERED,
      CompositorEventType.VIDEO_INPUT_PLAYING,
      CompositorEventType.AUDIO_INPUT_PLAYING,
      CompositorEventType.VIDEO_INPUT_EOS,
      CompositorEventType.AUDIO_INPUT_EOS,
    ].includes(event.type)
  ) {
    return { type: event.type, inputId: event.input_id };
  } else if (CompositorEventType.OUTPUT_DONE === event.type) {
    return { type: event.type, outputId: event.outputId };
  } else {
    console.error(`Unknown event type: ${event.type}`);
    return null;
  }
}
