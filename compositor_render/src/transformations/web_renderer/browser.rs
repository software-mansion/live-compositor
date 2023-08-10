use compositor_chromium::cef;
use compositor_common::scene::Resolution;
use crossbeam_channel::{Receiver, Sender};

pub(super) struct BrowserState {
    frame_rx: Receiver<Vec<u8>>,
    frame_data: Vec<u8>,
}

impl BrowserState {
    pub fn new(frame_rx: Receiver<Vec<u8>>) -> Self {
        Self {
            frame_rx,
            frame_data: Vec::new(),
        }
    }

    pub fn retrieve_frame(&mut self) -> &[u8] {
        if let Ok(frame) = self.frame_rx.try_recv() {
            self.frame_data = frame;
        }

        &self.frame_data
    }
}

pub struct BrowserClient {
    frame_tx: Sender<Vec<u8>>,
    resolution: Resolution,
}

impl cef::Client for BrowserClient {
    type RenderHandlerType = RenderHandler;

    fn render_handler(&self) -> Option<Self::RenderHandlerType> {
        Some(RenderHandler::new(self.frame_tx.clone(), self.resolution))
    }
}

impl BrowserClient {
    pub fn new(frame_tx: Sender<Vec<u8>>, resolution: Resolution) -> Self {
        Self {
            frame_tx,
            resolution,
        }
    }
}

pub struct RenderHandler {
    frame_tx: Sender<Vec<u8>>,
    resolution: Resolution,
}

impl cef::RenderHandler for RenderHandler {
    fn resolution(&self, _browser: &cef::Browser) -> Resolution {
        self.resolution
    }

    fn on_paint(&self, _browser: &cef::Browser, buffer: &[u8], _resolution: Resolution) {
        if !self.frame_tx.is_full() {
            self.frame_tx.send(buffer.to_vec()).expect("send frame");
        }
    }
}

impl RenderHandler {
    pub fn new(frame_tx: Sender<Vec<u8>>, resolution: Resolution) -> Self {
        Self {
            frame_tx,
            resolution,
        }
    }
}
