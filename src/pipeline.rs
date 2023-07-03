use anyhow::{anyhow, Result};
use std::{
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    decoder::Decoder,
    encoder::Encoder,
    rtp::{RtpFrame, RtpPacker, RtpParser},
    state::{Frame, SyncHashMap},
};

pub struct Pipeline {
    inputs: SyncHashMap<u32, Arc<PipelineInput>>,
    outputs: SyncHashMap<u32, Arc<PipelineOutput>>,
    //queue: LiveQueue,
    //renderer: Renderer,
}

#[allow(dead_code)]
pub struct PipelineOutput {
    encoder: Encoder,
    rtp_packer: RtpPacker,
    transport: Mutex<Sender<bytes::Bytes>>,
}

pub struct PipelineInput {
    decoder: Decoder,
    rtp_parser: RtpParser,
}

// unpack rtp -> decode -> queue -> render -> encode -> pack rtp
// this code would be in a separate crate (render in this scenario would also be a separate crate)
impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            inputs: SyncHashMap::new(),
            outputs: SyncHashMap::new(),
        }
    }

    pub fn add_input(&self, input_id: u32) {
        self.inputs.insert(
            input_id,
            Arc::new(PipelineInput {
                decoder: Decoder::new(),
                rtp_parser: RtpParser::new(),
            }),
        );
    }

    pub fn add_output(&self, input_id: u32, transport: Sender<bytes::Bytes>) {
        self.outputs.insert(
            input_id,
            Arc::new(PipelineOutput {
                encoder: Encoder::new(),
                rtp_packer: RtpPacker::new(),
                transport: Mutex::new(transport),
            }),
        );
    }

    pub fn push_input_data(&self, input_id: u32, buffer: bytes::Bytes) -> Result<()> {
        let input = self
            .inputs
            .get_cloned(&input_id)
            .ok_or_else(|| anyhow!("no input with id {}.", input_id))?;
        input.process_input(buffer)?;
        //self.queue.enqueue(input_id, frames);
        Ok(())
    }

    #[allow(dead_code)]
    fn on_output_data_received(&self, output_id: u32, frames: Vec<Frame>) {
        let result = self
            .outputs
            .get_cloned(&output_id)
            .ok_or_else(|| anyhow!("no output with id {}.", output_id))
            .and_then(|output| output.process_output(frames));
        if let Err(err) = result {
            eprintln!("{}", err)
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
                eprintln!("render loop");
                thread::sleep(Duration::from_millis(1000));
            }
        });
    }
}

impl PipelineInput {
    fn process_input(&self, buffer: bytes::Bytes) -> Result<Vec<Frame>> {
        self.rtp_parser
            .parse(buffer)?
            .into_iter()
            .flat_map(|packet| match self.decoder.decode(packet.data) {
                Ok(value) => value
                    .into_iter()
                    .map(|data| Ok(Frame { data }))
                    .collect::<Vec<Result<Frame>>>(),
                Err(e) => vec![Err(e)],
            })
            .collect()
    }
}

impl PipelineOutput {
    fn process_output(&self, frames: Vec<Frame>) -> Result<()> {
        let encoded = self.encoder.encode(frames.into_iter().map(|i| i.data))?;

        let raw_data = self
            .rtp_packer
            .pack(encoded.into_iter().map(|i| RtpFrame { data: i }))?;
        let raw_data = match raw_data {
            Some(data) => data,
            None => return Ok(()),
        };

        self.transport.lock().unwrap().send(raw_data)?;
        Ok(())
    }
}
