use std::sync::Arc;

use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};

use crate::{
    scene::ComponentId, transformations::layout::transformation_matrices::Position, Resolution,
};

pub const EMBED_FRAMES_MESSAGE: &str = "EMBED_FRAMES";
pub const DROP_SHARED_MEMORY: &str = "DROP_SHARED_MEMORY";
pub const GET_FRAME_POSITIONS_MESSAGE: &str = "GET_FRAME_POSITIONS";

pub struct ResponseSender<Response> {
    sender: Option<Sender<Response>>,
}

impl<Response> ResponseSender<Response> {
    pub fn new(sender: Sender<Response>) -> Self {
        Self {
            sender: Some(sender),
        }
    }

    pub fn send(mut self, response: Response) -> Result<(), ResponseSenderError> {
        match self.sender.take() {
            Some(sender) => sender
                .send(response)
                .map_err(|_| ResponseSenderError::SendFailed),
            None => Err(ResponseSenderError::AlreadySent),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResponseSenderError {
    #[error("Response sender has already been used")]
    AlreadySent,

    #[error("Failed to send response")]
    SendFailed,
}

pub struct ResponseReceiver<Response> {
    receiver: Option<Receiver<Response>>,
}

impl<Response> ResponseReceiver<Response> {
    pub fn new(receiver: Receiver<Response>) -> Self {
        Self {
            receiver: Some(receiver),
        }
    }

    pub fn recv(mut self) -> Result<Response, ResponseReceiverError> {
        match self.receiver.take() {
            Some(receiver) => receiver
                .recv()
                .map_err(|_| ResponseReceiverError::ReceiveFailed),
            None => Err(ResponseReceiverError::AlreadyResponded),
        }
    }
}

pub fn new_response_channel<Response>() -> (ResponseSender<Response>, ResponseReceiver<Response>) {
    let (sender, receiver) = crossbeam_channel::bounded(1);
    (ResponseSender::new(sender), ResponseReceiver::new(receiver))
}

#[derive(Debug, thiserror::Error)]
pub enum ResponseReceiverError {
    #[error("Response receiver has already been used")]
    AlreadyResponded,

    #[error("Failed to receive response")]
    ReceiveFailed,
}

pub enum WebRendererThreadRequest {
    GetRenderedWebsite {
        response_sender: ResponseSender<Option<Bytes>>,
    },
    EmbedSources {
        resolutions: Vec<Option<Resolution>>,
        children_ids: Vec<ComponentId>,
    },
    EnsureSharedMemory {
        resolutions: Vec<Option<Resolution>>,
    },
    UpdateSharedMemory {
        payload: UpdateSharedMemoryPayload,
        response_sender: ResponseSender<()>,
    },
    GetFramePositions {
        children_ids: Vec<ComponentId>,
        response_sender: ResponseSender<Vec<Position>>,
    },
    Quit,
}

pub struct UpdateSharedMemoryPayload {
    pub source_idx: usize,
    pub buffer: Arc<wgpu::Buffer>,
    pub size: wgpu::Extent3d,
}
