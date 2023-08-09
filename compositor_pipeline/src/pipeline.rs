use std::collections::{hash_map, HashMap};
use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::{ops::Deref, sync::Arc};

use compositor_common::scene::{InputId, OutputId, SceneSpec};
use compositor_common::transformation::{TransformationRegistryKey, TransformationSpec};
use compositor_common::{Frame, Framerate};
use compositor_render::renderer::{RendererNewError, RendererRegisterTransformationError};
use compositor_render::{renderer::scene::SceneUpdateError, Renderer};
use crossbeam_channel::unbounded;
use log::error;
use serde::{Deserialize, Serialize};

use crate::queue::Queue;

pub trait PipelineOutput: Send + Sync + 'static {
    type Opts;

    fn send_frame(&self, frame: Frame);
    fn new(opts: Self::Opts) -> Self;
}

pub trait PipelineInput: Send + Sync + 'static {
    type Opts;

    fn new(queue: Arc<Queue>, opts: Self::Opts) -> Self;
}

pub struct Pipeline<Input: PipelineInput, Output: PipelineOutput> {
    inputs: HashMap<InputId, Arc<Input>>,
    outputs: OutputRegistry<Output>,
    queue: Arc<Queue>,
    renderer: Renderer,
}

#[derive(Serialize, Deserialize)]
pub struct Options {
    pub framerate: Framerate,
    pub init_web_renderer: Option<bool>,
}

impl<Input: PipelineInput, Output: PipelineOutput> Pipeline<Input, Output> {
    pub fn new(opts: Options) -> Result<Self, RendererNewError> {
        Ok(Pipeline {
            outputs: OutputRegistry::new(),
            inputs: HashMap::new(),
            queue: Arc::new(Queue::new(opts.framerate)),
            renderer: Renderer::new(opts.init_web_renderer.unwrap_or(true))?,
        })
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
        if scene_spec
            .inputs
            .iter()
            .any(|node| &node.input_id == input_id)
        {
            return Err(UnregisterInputError::StillInUse(input_id.clone()));
        }

        self.inputs.remove(input_id);
        self.queue.remove_input(input_id);
        Ok(())
    }

    pub fn register_output(
        &self,
        output_id: OutputId,
        output_opts: Output::Opts,
    ) -> Result<(), RegisterOutputError> {
        if self.outputs.contains_key(&output_id) {
            return Err(RegisterOutputError::AlreadyRegistered(output_id));
        }
        self.outputs
            .insert(output_id, Output::new(output_opts).into());
        Ok(())
    }

    pub fn unregister_output(&self, output_id: &OutputId) -> Result<(), UnregisterOutputError> {
        if !self.outputs.contains_key(output_id) {
            return Err(UnregisterOutputError::NotFound(output_id.clone()));
        }

        let scene_spec = self.renderer.scene_spec();
        if scene_spec
            .outputs
            .iter()
            .any(|node| &node.output_id == output_id)
        {
            return Err(UnregisterOutputError::StillInUse(output_id.clone()));
        }

        self.outputs.remove(output_id);
        Ok(())
    }

    pub fn register_transformation(
        &self,
        key: TransformationRegistryKey,
        transformation_spec: TransformationSpec,
    ) -> Result<(), RendererRegisterTransformationError> {
        self.renderer
            .register_transformation(key, transformation_spec)?;
        Ok(())
    }

    pub fn update_scene(&mut self, scene_spec: Arc<SceneSpec>) -> Result<(), SceneUpdateError> {
        scene_spec
            .validate(
                &self.inputs.keys().map(|i| &i.0).collect(),
                &self.outputs.lock().keys().map(|i| &i.0).collect(),
            )
            .map_err(SceneUpdateError::InvalidSpec)?;
        self.renderer.update_scene(scene_spec)
    }

    pub fn start(&self) {
        let (frames_sender, frames_receiver) = unbounded();
        let renderer = self.renderer.clone();
        let outputs = self.outputs.clone();

        self.queue.start(frames_sender);

        thread::spawn(move || loop {
            let input_frames = frames_receiver.recv().unwrap();
            if !input_frames.frames.is_empty() {
                let output = renderer.render(input_frames);

                for (id, frame) in &output.frames {
                    let output = outputs.lock().get(id).map(Clone::clone);
                    let Some(output) = output else {
                                error!("no output with id {}", id.0.0);
                                continue;
                        };

                    output.send_frame(frame.deref().clone());
                }
            }
        });
    }

    pub fn inputs(&self) -> impl Iterator<Item = (&InputId, &Input)> {
        self.inputs.iter().map(|(id, node)| (id, node.as_ref()))
    }

    // TODO: figure out how to pass Iterator<Item = (&OutputId, &Output)> instead
    pub fn with_outputs<'a, F, R>(&'a self, f: F) -> R
    where
        F: Fn(hash_map::Iter<'_, OutputId, Arc<Output>>) -> R + 'a,
    {
        let guard = self.outputs.lock();
        f(guard.iter())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterInputError {
    #[error("Input {0} is already registered.")]
    AlreadyRegistered(InputId),
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterOutputError {
    #[error("Output {0} is already registered.")]
    AlreadyRegistered(OutputId),
}

#[derive(Debug, thiserror::Error)]
pub enum UnregisterInputError {
    #[error("Input {0} does not exist.")]
    NotFound(InputId),

    #[error("Input {0} is still part of the scene.")]
    StillInUse(InputId),
}

#[derive(Debug, thiserror::Error)]
pub enum UnregisterOutputError {
    #[error("Output {0} does not exists.")]
    NotFound(OutputId),

    #[error("Output {0} is still part of the scene")]
    StillInUse(OutputId),
}

struct OutputRegistry<Output: PipelineOutput>(Arc<Mutex<HashMap<OutputId, Arc<Output>>>>);

impl<Output: PipelineOutput> Clone for OutputRegistry<Output> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Output: PipelineOutput> OutputRegistry<Output> {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    fn contains_key(&self, key: &OutputId) -> bool {
        self.0.lock().unwrap().contains_key(key)
    }

    fn insert(&self, key: OutputId, value: Arc<Output>) -> Option<Arc<Output>> {
        self.0.lock().unwrap().insert(key, value)
    }

    fn remove(&self, key: &OutputId) -> Option<Arc<Output>> {
        self.0.lock().unwrap().remove(key)
    }

    fn lock(&self) -> MutexGuard<HashMap<OutputId, Arc<Output>>> {
        self.0.lock().unwrap()
    }
}
