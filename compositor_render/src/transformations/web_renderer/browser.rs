use compositor_chromium::cef;
use compositor_common::scene::Resolution;
use crossbeam_channel::{Receiver, Sender};

pub(super) struct BrowserState {
    painted_frames_receiver: Receiver<Vec<u8>>,
    frame_data: Option<Vec<u8>>,
}

impl BrowserState {
    pub fn new(painted_frames_receiver: Receiver<Vec<u8>>) -> Self {
        Self {
            painted_frames_receiver,
            frame_data: None,
        }
    }

    pub fn retrieve_frame(&mut self) -> Option<&[u8]> {
        if let Some(frame) = self.painted_frames_receiver.try_iter().last() {
            self.frame_data.replace(frame);
        }

        self.frame_data.as_deref()
    }
}

pub struct BrowserClient {
    painted_frames_sender: Sender<Vec<u8>>,
    resolution: Resolution,
}

impl cef::Client for BrowserClient {
    type RenderHandlerType = RenderHandler;

    fn render_handler(&self) -> Option<Self::RenderHandlerType> {
        Some(RenderHandler::new(
            self.painted_frames_sender.clone(),
            self.resolution,
        ))
    }
}

impl BrowserClient {
    pub fn new(painted_frames_sender: Sender<Vec<u8>>, resolution: Resolution) -> Self {
        Self {
            painted_frames_sender,
            resolution,
        }
    }
}

pub struct RenderHandler {
    painted_frames_sender: Sender<Vec<u8>>,
    resolution: Resolution,
}

impl cef::RenderHandler for RenderHandler {
    fn resolution(&self, _browser: &cef::Browser) -> Resolution {
        self.resolution
    }

    fn on_paint(&self, _browser: &cef::Browser, buffer: &[u8], _resolution: Resolution) {
        self.painted_frames_sender
            .send(buffer.to_vec())
            .expect("send frame");
    }
}

impl RenderHandler {
    pub fn new(painted_frames_sender: Sender<Vec<u8>>, resolution: Resolution) -> Self {
        Self {
            painted_frames_sender,
            resolution,
        }
    }
}
