use std::collections::{hash_map, HashMap};
use std::sync::Arc;
use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use compositor_common::error::ErrorStack;
use compositor_common::renderer_spec::{RendererId, RendererSpec};
use compositor_common::scene::{InputId, OutputId, Resolution, SceneSpec};
use compositor_common::{Frame, Framerate};
use compositor_render::error::{
    InitRendererEngineError, RegisterRendererError, UnregisterRendererError,
};
use compositor_render::event_loop::EventLoop;
use compositor_render::registry::RegistryType;
use compositor_render::renderer::RendererOptions;
use compositor_render::WebRendererOptions;
use compositor_render::{error::UpdateSceneError, Renderer};
use crossbeam_channel::unbounded;
use crossbeam_channel::Sender;
use ffmpeg_next::{Codec, Packet};
use log::{error, warn};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationMilliSeconds};

use crate::error::{
    RegisterInputError, RegisterOutputError, UnregisterInputError, UnregisterOutputError,
};
use crate::queue::Queue;

use self::encoder::{Encoder, EncoderSettings};

pub mod encoder;

pub trait PipelineOutputReceiver: 'static {
    type Opts: Send + Sync + 'static;
    type Identifier: Send + Sync + 'static;

    fn send_packet(&mut self, packet: Packet);
    fn new(opts: Self::Opts, codec: Codec) -> Self;
    fn identifier(&self) -> Self::Identifier;
}

pub trait PipelineInput: Send + Sync + 'static {
    type Opts;

    fn new(queue: Arc<Queue>, opts: Self::Opts) -> Self;
}

pub struct OutputOptions<Receiver: PipelineOutputReceiver> {
    pub receiver_options: Receiver::Opts,
    pub encoder_settings: EncoderSettings,
    pub resolution: Resolution,
}

pub struct PipelineOutput<Receiver: PipelineOutputReceiver> {
    sender: Sender<Frame>,
    receiver_identifier: Receiver::Identifier,
}

impl<Receiver: PipelineOutputReceiver> PipelineOutput<Receiver> {
    fn new(opts: OutputOptions<Receiver>) -> Result<Self, String> {
        let mut encoder = Encoder::new(opts.encoder_settings, opts.resolution)?;
        let (tx, rx) = crossbeam_channel::unbounded();
        let (id_tx, id_rx) = crossbeam_channel::bounded(1);

        std::thread::spawn(move || {
            let mut receiver = Receiver::new(opts.receiver_options, encoder.codec());
            id_tx.send(receiver.identifier()).unwrap();
            for frame in rx.iter() {
                if rx.len() > 20 {
                    warn!("Dropping frame: encoder queue is too long.");
                    continue;
                }

                for packet in encoder.send_frame(frame) {
                    receiver.send_packet(packet);
                }
            }
        });

        Ok(Self {
            sender: tx,
            receiver_identifier: id_rx.recv().unwrap(),
        })
    }

    fn send_frame(&self, frame: Frame) {
        self.sender.send(frame).unwrap();
    }

    pub fn identifier(&self) -> &Receiver::Identifier {
        &self.receiver_identifier
    }
}

pub struct Pipeline<Input: PipelineInput, Receiver: PipelineOutputReceiver> {
    inputs: HashMap<InputId, Arc<Input>>,
    outputs: OutputRegistry<PipelineOutput<Receiver>>,
    queue: Arc<Queue>,
    renderer: Renderer,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Options {
    pub framerate: Framerate,
    #[serde_as(as = "Option<DurationMilliSeconds<f64>>")]
    #[serde(rename = "stream_fallback_timeout_ms")]
    pub stream_fallback_timeout: Option<Duration>,
    #[serde(default)]
    pub web_renderer: WebRendererOptions,
}

impl<Input: PipelineInput, Receiver: PipelineOutputReceiver> Pipeline<Input, Receiver> {
    pub fn new(opts: Options) -> Result<(Self, EventLoop), InitRendererEngineError> {
        let (renderer, event_loop) = Renderer::new(RendererOptions {
            web_renderer: opts.web_renderer,
            framerate: opts.framerate,
            stream_fallback_timeout: opts
                .stream_fallback_timeout
                .unwrap_or(Duration::from_secs(1)),
        })?;
        let pipeline = Pipeline {
            outputs: OutputRegistry::new(),
            inputs: HashMap::new(),
            queue: Arc::new(Queue::new(opts.framerate)),
            renderer,
        };

        Ok((pipeline, event_loop))
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn register_input(
        &mut self,
        input_id: InputId,
        input_opts: Input::Opts,
    ) -> Result<(), RegisterInputError> {
        if self.inputs.contains_key(&input_id) {
            return Err(RegisterInputError::AlreadyRegistered(input_id));
        }

        self.inputs.insert(
            input_id.clone(),
            Input::new(self.queue.clone(), input_opts).into(),
        );
        self.queue.add_input(input_id);
        Ok(())
    }

    pub fn unregister_input(&mut self, input_id: &InputId) -> Result<(), UnregisterInputError> {
        if !self.inputs.contains_key(input_id) {
            return Err(UnregisterInputError::NotFound(input_id.clone()));
        }

        let scene_spec = self.renderer.scene_spec();
        let is_still_in_use = scene_spec
            .nodes
            .iter()
            .flat_map(|node| &node.input_pads)
            .any(|input_pad_id| &input_id.0 == input_pad_id);
        if is_still_in_use {
            return Err(UnregisterInputError::StillInUse(input_id.clone()));
        }

        self.inputs.remove(input_id);
        self.queue.remove_input(input_id);
        Ok(())
    }

    pub fn register_output(
        &self,
        output_id: OutputId,
        output_opts: OutputOptions<Receiver>,
    ) -> Result<(), RegisterOutputError> {
        if self.outputs.contains_key(&output_id) {
            return Err(RegisterOutputError::AlreadyRegistered(output_id));
        }

        let output = PipelineOutput::new(output_opts)
            .map_err(|e| RegisterOutputError::EncoderError(output_id.clone(), e))?;

        self.outputs.insert(output_id, output.into());
        Ok(())
    }

    pub fn unregister_output(&self, output_id: &OutputId) -> Result<(), UnregisterOutputError> {
        if !self.outputs.contains_key(output_id) {
            return Err(UnregisterOutputError::NotFound(output_id.clone()));
        }

        let scene_spec = self.renderer.scene_spec();
        let is_still_in_use = scene_spec
            .outputs
            .iter()
            .any(|node| &node.output_id == output_id);
        if is_still_in_use {
            return Err(UnregisterOutputError::StillInUse(output_id.clone()));
        }

        self.outputs.remove(output_id);
        Ok(())
    }

    pub fn register_renderer(
        &self,
        transformation_spec: RendererSpec,
    ) -> Result<(), RegisterRendererError> {
        self.renderer.register_renderer(transformation_spec)?;
        Ok(())
    }

    pub fn unregister_renderer(
        &self,
        renderer_id: &RendererId,
        registry_type: RegistryType,
    ) -> Result<(), UnregisterRendererError> {
        self.renderer
            .unregister_renderer(renderer_id, registry_type)
    }

    pub fn update_scene(&mut self, scene_spec: Arc<SceneSpec>) -> Result<(), UpdateSceneError> {
        scene_spec
            .validate(
                &self.inputs.keys().map(|i| &i.0).collect(),
                &self.outputs.lock().keys().map(|i| &i.0).collect(),
            )
            .map_err(UpdateSceneError::InvalidSpec)?;
        self.renderer.update_scene(scene_spec)
    }

    pub fn start(&self) {
        let (frames_sender, frames_receiver) = unbounded();
        let renderer = self.renderer.clone();
        let outputs = self.outputs.clone();

        self.queue.start(frames_sender);

        thread::spawn(move || {
            for input_frames in frames_receiver.iter() {
                if frames_receiver.len() > 20 {
                    warn!("Dropping frame: render queue is too long.",);
                    continue;
                }

                let output = renderer.render(input_frames);
                let Ok(output_frames) = output else {
                    error!(
                        "Error while rendering: {}",
                        ErrorStack::new(&output.unwrap_err()).into_string()
                    );
                    continue;
                };

                for (id, frame) in output_frames.frames {
                    let output = outputs.lock().get(&id).map(Clone::clone);
                    let Some(output) = output else {
                        error!("no output with id {}", &id);
                        continue;
                    };

                    output.send_frame(frame);
                }
            }
        });
    }

    pub fn inputs(&self) -> impl Iterator<Item = (&InputId, &Input)> {
        self.inputs.iter().map(|(id, node)| (id, node.as_ref()))
    }

    pub fn with_outputs<F, R>(&self, f: F) -> R
    where
        F: Fn(OutputIterator<'_, PipelineOutput<Receiver>>) -> R,
    {
        let guard = self.outputs.lock();
        f(OutputIterator::new(guard.iter()))
    }
}

struct OutputRegistry<T>(Arc<Mutex<HashMap<OutputId, Arc<T>>>>);

impl<T> Clone for OutputRegistry<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> OutputRegistry<T> {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    fn contains_key(&self, key: &OutputId) -> bool {
        self.0.lock().unwrap().contains_key(key)
    }

    fn insert(&self, key: OutputId, value: Arc<T>) -> Option<Arc<T>> {
        self.0.lock().unwrap().insert(key, value)
    }

    fn remove(&self, key: &OutputId) -> Option<Arc<T>> {
        self.0.lock().unwrap().remove(key)
    }

    fn lock(&self) -> MutexGuard<HashMap<OutputId, Arc<T>>> {
        self.0.lock().unwrap()
    }
}

pub struct OutputIterator<'a, T> {
    inner_iter: hash_map::Iter<'a, OutputId, Arc<T>>,
}

impl<'a, T> OutputIterator<'a, T> {
    fn new(iter: hash_map::Iter<'a, OutputId, Arc<T>>) -> Self {
        Self { inner_iter: iter }
    }
}

impl<'a, T> Iterator for OutputIterator<'a, T> {
    type Item = (&'a OutputId, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter.next().map(|(id, node)| (id, node.as_ref()))
    }
}
