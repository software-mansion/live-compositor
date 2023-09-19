use std::sync::{Arc, Mutex};

use crate::GET_FRAME_POSITIONS_MESSAGE;
use bytes::{Bytes, BytesMut};
use compositor_chromium::cef;
use compositor_common::scene::Resolution;
use log::error;

use crate::transformations::builtin::box_layout::BoxLayout;
use crate::transformations::builtin::utils::mat4_to_bytes;
use crate::transformations::web_renderer::frame_embedder::FrameTransforms;
use crate::transformations::web_renderer::FrameBytes;

#[derive(Clone)]
pub(super) struct BrowserClient {
    frame_bytes: FrameBytes,
    frame_transforms: FrameTransforms,
    resolution: Resolution,
}

impl cef::Client for BrowserClient {
    type RenderHandlerType = RenderHandler;

    fn render_handler(&self) -> Option<Self::RenderHandlerType> {
        Some(RenderHandler::new(
            self.frame_bytes.clone(),
            self.resolution,
        ))
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
                let mut transformations_matrices = BytesMut::new();
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

                    let transformations_matrix = BoxLayout {
                        top_left_corner: (x as f32, y as f32),
                        width: width as f32,
                        height: height as f32,
                        rotation_degrees: 0.0,
                    }
                    .transformation_matrix(self.resolution);

                    transformations_matrices.extend(mat4_to_bytes(&transformations_matrix));
                }

                let mut frame_positions = self.frame_transforms.lock().unwrap();
                *frame_positions = transformations_matrices.freeze();
            }
            ty => error!("Unknown process message type \"{ty}\""),
        }
        true
    }
}

impl BrowserClient {
    pub fn new(
        frame_bytes: FrameBytes,
        frame_transforms: FrameTransforms,
        resolution: Resolution,
    ) -> Self {
        Self {
            frame_bytes,
            frame_transforms,
            resolution,
        }
    }
}

pub(super) struct RenderHandler {
    frame_bytes: FrameBytes,
    resolution: Resolution,
}

impl cef::RenderHandler for RenderHandler {
    fn resolution(&self, _browser: &cef::Browser) -> Resolution {
        self.resolution
    }

    fn on_paint(&self, _browser: &cef::Browser, buffer: &[u8], _resolution: Resolution) {
        let mut frame_bytes = self.frame_bytes.lock().unwrap();
        *frame_bytes = Bytes::copy_from_slice(buffer);
    }
}

impl RenderHandler {
    pub fn new(frame_bytes: Arc<Mutex<Bytes>>, resolution: Resolution) -> Self {
        Self {
            frame_bytes,
            resolution,
        }
    }
}
