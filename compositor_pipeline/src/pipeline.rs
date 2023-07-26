use compositor_common::{Frame, Framerate, InputId};
use compositor_render::renderer::Renderer;
use crossbeam_channel::unbounded;
use log::error;
use std::{sync::Arc, thread};

use crate::{
    map::SyncHashMap,
    queue::{Queue, QueueError},
};

pub trait PipelineOutput {
    fn send_frame(&self, frame: Frame);
}

pub struct Pipeline<Output: PipelineOutput> {
    outputs: SyncHashMap<u32, Arc<Output>>,
    queue: Queue,
}

impl<Output: PipelineOutput + std::marker::Send + std::marker::Sync + 'static> Pipeline<Output> {
    pub fn new(framerate: Framerate) -> Self {
        Pipeline {
            outputs: SyncHashMap::new(),
            queue: Queue::new(framerate),
        }
    }

    pub fn add_input(&self, input_id: InputId) {
        self.queue.add_input(input_id);
        // self.renderer.add_input();
    }

    pub fn add_output(&self, output_id: u32, output: Arc<Output>) {
        self.outputs.insert(output_id, output);
    }

    pub fn push_input_data(&self, input_id: InputId, frame: Frame) -> Result<(), QueueError> {
        self.queue.enqueue_frame(input_id, frame)
    }

    #[allow(dead_code)]
    fn on_output_data_received(&self, output_id: u32, frame: Frame) {
        match self.outputs.get_cloned(&output_id) {
            Some(output) => output.send_frame(frame),
            None => {
                error!("Output {} not found", output_id);
            }
        }
    }

    pub fn start(self: &Arc<Self>) {
        let (frames_sender, frames_receiver) = unbounded();
        let pipeline = self.clone();

        pipeline.queue.start(frames_sender);

        thread::spawn(move || {
            // TODO move it to pipeline - fix Send, Sync stuff
            let renderer = Renderer::new().unwrap();
            loop {
                let input_frames = frames_receiver.recv().unwrap();
                if !input_frames.frames.is_empty() {
                    let output_frame = renderer.render(input_frames).unwrap();
                    pipeline
                        .outputs
                        .get_cloned(&8002)
                        .unwrap()
                        .send_frame(output_frame);
                }
            }
        });
    }
}

impl<Output: PipelineOutput + std::marker::Send + std::marker::Sync + 'static> Default
    for Pipeline<Output>
{
    fn default() -> Self {
        Self::new(Framerate(30))
    }
}
