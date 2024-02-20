use std::sync::{Arc, Mutex};

use crate::{
    transformations::{
        layout::transformation_matrices::Position,
        web_renderer::{DROP_SHARED_MEMORY, GET_FRAME_POSITIONS_MESSAGE},
    },
    Resolution,
};
use bytes::Bytes;
use compositor_chromium::cef::{self, ProcessMessageError};
use crossbeam_channel::Sender;
use log::error;

#[derive(Clone)]
pub(super) struct BrowserClient {
    frame_data: Arc<Mutex<Bytes>>,
    frame_positions_sender: Sender<Vec<Position>>,
    shared_memory_dropped_sender: Sender<()>,
    resolution: Resolution,
}

impl cef::Client for BrowserClient {
    type RenderHandlerType = RenderHandler;

    fn render_handler(&self) -> Option<Self::RenderHandlerType> {
        Some(RenderHandler::new(self.frame_data.clone(), self.resolution))
    }

    fn on_process_message_received(
        &mut self,
        _browser: &cef::Browser,
        _frame: &cef::Frame,
        _source_process: cef::ProcessId,
        message: &cef::ProcessMessage,
    ) -> bool {
        match message.name().as_str() {
            GET_FRAME_POSITIONS_MESSAGE => {
                let frame_positions = match self.read_frame_positions(message) {
                    Ok(frame_positions) => frame_positions,
                    Err(err) => {
                        error!("Failed to read frame positions: {err}");
                        return true;
                    }
                };
                if self.frame_positions_sender.send(frame_positions).is_err() {
                    error!("Failed to send frame positions");
                }
            }
            DROP_SHARED_MEMORY => {
                if self.shared_memory_dropped_sender.send(()).is_err() {
                    error!("Failed to send shared memory dropped confirmation");
                }
            }
            ty => error!("Unknown process message type \"{ty}\""),
        }
        true
    }
}

impl BrowserClient {
    pub fn new(
        frame_data: Arc<Mutex<Bytes>>,
        frame_positions_sender: Sender<Vec<Position>>,
        shared_memory_dropped_sender: Sender<()>,
        resolution: Resolution,
    ) -> Self {
        Self {
            frame_data,
            frame_positions_sender,
            shared_memory_dropped_sender,
            resolution,
        }
    }

    fn read_frame_positions(
        &self,
        message: &cef::ProcessMessage,
    ) -> Result<Vec<Position>, ProcessMessageError> {
        let mut frame_positions = Vec::new();
        for i in (0..message.size()).step_by(4) {
            let x = message.read_double(i)?;
            let y = message.read_double(i + 1)?;
            let width = message.read_double(i + 2)?;
            let height = message.read_double(i + 3)?;

            frame_positions.push(Position {
                top: y as f32,
                left: x as f32,
                width: width as f32,
                height: height as f32,
                rotation_degrees: 0.0,
            });
        }

        Ok(frame_positions)
    }
}

pub(super) struct RenderHandler {
    frame_data: Arc<Mutex<Bytes>>,
    resolution: Resolution,
}

impl cef::RenderHandler for RenderHandler {
    fn resolution(&self, _browser: &cef::Browser) -> cef::Resolution {
        cef::Resolution {
            width: self.resolution.width,
            height: self.resolution.height,
        }
    }

    fn on_paint(&self, _browser: &cef::Browser, buffer: &[u8], _resolution: cef::Resolution) {
        let mut frame_data = self.frame_data.lock().unwrap();
        *frame_data = Bytes::copy_from_slice(buffer);
    }
}

impl RenderHandler {
    pub fn new(frame_data: Arc<Mutex<Bytes>>, resolution: Resolution) -> Self {
        Self {
            frame_data,
            resolution,
        }
    }
}
