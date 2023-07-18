use compositor_common::{frame::Framerate, Frame};
use compositor_render::renderer::Renderer;
use crossbeam_channel::unbounded;
use log::{error, info};
use std::{sync::Arc, thread};

use crate::{map::SyncHashMap, queue::Queue};

pub trait PipelineOutput {
    fn send_frame(&self, frame: Arc<Frame>);
}

pub struct Pipeline<Output: PipelineOutput> {
    outputs: SyncHashMap<u32, Arc<Output>>,
    queue: Queue,
}

impl<Output: PipelineOutput + std::marker::Send + std::marker::Sync + 'static> Pipeline<Output> {
    pub fn new() -> Self {
        Pipeline {
            outputs: SyncHashMap::new(),
            queue: Queue::new(Framerate(24)),
        }
    }

    pub fn add_input(&self, input_id: u32) {
        self.queue.add_input(input_id);
        // self.renderer.add_input();
    }

    pub fn add_output(&self, output_id: u32, output: Arc<Output>) {
        self.outputs.insert(output_id, output);
    }

    pub fn push_input_data(&self, input_id: u32, frame: Frame) {
        self.queue.enqueue_frame(input_id, frame).unwrap();
        // self.outputs.get_cloned(&8002).unwrap().send_frame(frame);
    }

    #[allow(dead_code)]
    fn on_output_data_received(&self, output_id: u32, frame: Arc<Frame>) {
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
                info!("render loop");
                let input_frames = frames_receiver.recv().unwrap();
                if !input_frames.frames.is_empty() {
                    info!("Input frames: {:#?}", input_frames.pts);
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
        Self::new()
    }
}
