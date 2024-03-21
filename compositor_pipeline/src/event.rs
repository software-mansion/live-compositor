use compositor_render::{event_handler, InputId};

pub(crate) enum Event {
    AudioInputStreamDelivered(InputId),
    VideoInputStreamDelivered(InputId),
    AudioInputStreamPlaying(InputId),
    VideoInputStreamPlaying(InputId),
    AudioInputStreamEos(InputId),
    VideoInputStreamEos(InputId),
}

fn input_event(kind: &str, input_id: InputId) -> event_handler::Event {
    event_handler::Event {
        kind: kind.to_string(),
        properties: vec![("input_id".to_string(), input_id.to_string())],
    }
}

impl From<Event> for event_handler::Event {
    fn from(val: Event) -> Self {
        match val {
            Event::AudioInputStreamDelivered(id) => input_event("AUDIO_INPUT_STREAM_DELIVERED", id),
            Event::VideoInputStreamDelivered(id) => input_event("VIDEO_INPUT_STREAM_DELIVERED", id),
            Event::AudioInputStreamPlaying(id) => input_event("AUDIO_INPUT_STREAM_PLAYING", id),
            Event::VideoInputStreamPlaying(id) => input_event("VIDEO_INPUT_STREAM_PLAYING", id),
            Event::AudioInputStreamEos(id) => input_event("AUDIO_INPUT_STREAM_EOS", id),
            Event::VideoInputStreamEos(id) => input_event("VIDEO_INPUT_STREAM_EOS", id),
        }
    }
}
