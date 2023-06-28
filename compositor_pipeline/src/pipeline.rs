use compositor_common::Frame;
use std::{sync::Arc, thread, time::Duration};

use crate::map::SyncHashMap;

pub trait PipelineSource {
    fn send_frame(&self, frame: Frame);
}

pub struct Pipeline<Source: PipelineSource> {
    outputs: SyncHashMap<u32, Arc<Source>>,
    //queue: LiveQueue,
    //renderer: Renderer,
}

impl<Source: PipelineSource> Pipeline<Source> {
    pub fn new() -> Self {
        Pipeline {
            outputs: SyncHashMap::new(),
        }
    }

    pub fn add_input(&self, _input_id: u32) {
        // self.queue.add_input();
        // self.renderer.add_input();
    }

    pub fn add_output(&self, input_id: u32, source: Arc<Source>) {
        self.outputs.insert(input_id, source);
    }

    pub fn push_input_data(&self, _input_id: u32, frame: Frame) {
        //self.queue.enqueue(input_id, frames);
        self.outputs.get_cloned(&8002).unwrap().send_frame(frame);
    }

    #[allow(dead_code)]
    fn on_output_data_received(&self, output_id: u32, frame: Frame) {
        let output = self.outputs.get_cloned(&output_id);
        let Some(output) = output else {
            eprintln!("Output {} not found", output_id);
            return;
        };
        output.send_frame(frame);
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
                eprintln!("render loop");
                thread::sleep(Duration::from_millis(1000));
            }
        });
    }
}

impl<Source: PipelineSource> Default for Pipeline<Source> {
    fn default() -> Self {
        Self::new()
    }
}
