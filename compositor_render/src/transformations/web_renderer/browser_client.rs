use std::sync::{Arc, Mutex};

use crate::{
    transformations::layout::{vertices_transformation_matrix, Position},
    Resolution, GET_FRAME_POSITIONS_MESSAGE,
};
use bytes::Bytes;
use compositor_chromium::cef;
use log::error;

use crate::transformations::web_renderer::{FrameData, SourceTransforms};

#[derive(Clone)]
pub(super) struct BrowserClient {
    frame_data: FrameData,
    source_transforms: SourceTransforms,
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
                let mut transforms_matrices = Vec::new();
                for i in (0..message.size()).step_by(4) {
                    let Some(x) = message.read_double(i) else {
                        error!("Expected \"x\" value of DOMRect");
                        continue;
                    };
                    let Some(y) = message.read_double(i + 1) else {
                        error!("Expected \"y\" value of DOMRect");
                        continue;
                    };
                    let Some(width) = message.read_double(i + 2) else {
                        error!("Expected \"width\" value of DOMRect");
                        continue;
                    };
                    let Some(height) = message.read_double(i + 3) else {
                        error!("Expected \"height\" value of DOMRect");
                        continue;
                    };

                    let transformations_matrix = vertices_transformation_matrix(
                        &Position {
                            top: y as f32,
                            left: x as f32,
                            width: width as f32,
                            height: height as f32,
                            rotation_degrees: 0.0,
                        },
                        &self.resolution,
                    );

                    transforms_matrices.push(transformations_matrix);
                }

                let mut source_transforms = self.source_transforms.lock().unwrap();
                *source_transforms = transforms_matrices;
            }
            ty => error!("Unknown process message type \"{ty}\""),
        }
        true
    }
}

impl BrowserClient {
    pub fn new(
        frame_data: FrameData,
        source_transforms: SourceTransforms,
        resolution: Resolution,
    ) -> Self {
        Self {
            frame_data,
            source_transforms,
            resolution,
        }
    }
}

pub(super) struct RenderHandler {
    frame_data: FrameData,
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
