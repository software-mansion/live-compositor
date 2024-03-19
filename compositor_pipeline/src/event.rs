use compositor_render::{event_handler, InputId};

pub(crate) enum Event {
    AudioInputStreamDelivered(InputId),
    VideoInputStreamDelivered(InputId),
    AudioInputStreamPlaying(InputId),
    VideoInputStreamPlaying(InputId),
    AudioInputStreamEos(InputId),
    VideoInputStreamEos(InputId),
}

impl Into<event_handler::Event> for Event {
    fn into(self) -> event_handler::Event {
        todo!()
    }
}
