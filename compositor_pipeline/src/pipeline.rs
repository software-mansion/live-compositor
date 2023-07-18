use compositor_common::Frame;
use log::{error, info};
use std::{sync::Arc, thread, time::Duration};

use crate::map::SyncHashMap;

pub trait PipelineOutput {
    fn send_frame(&self, frame: Frame);
}

pub struct Pipeline<Output: PipelineOutput> {
    outputs: SyncHashMap<u32, Arc<Output>>,
    //queue: LiveQueue,
    //renderer: Renderer,
}

impl<Output: PipelineOutput> Pipeline<Output> {
    pub fn new() -> Self {
        Pipeline {
            outputs: SyncHashMap::new(),
        }
    }

    pub fn add_input(&self, _input_id: u32) {
        // self.queue.add_input();
        // self.renderer.add_input();
    }

    pub fn add_output(&self, output_id: u32, output: Arc<Output>) {
        self.outputs.insert(output_id, output);
    }

    pub fn push_input_data(&self, _input_id: u32, frame: Frame) {
        //self.queue.enqueue(input_id, frames);
        self.outputs.get_cloned(&8002).unwrap().send_frame(frame);
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
        let _pipeline = self.clone();
        thread::spawn(|| {
            loop {
                // probably sth like this
                //
                // let input_frames = pipeline.queue.next();
                // let pipeline.render.render(output_frames);
                // for let (output_id, frames) in input_frames {
                //     self.on_output_data_received(output_id, frames)
                // }
                info!("render loop");
                thread::sleep(Duration::from_millis(1000));
            }
        });
    }
}

impl<Output: PipelineOutput> Default for Pipeline<Output> {
    fn default() -> Self {
        Self::new()
    }
}
