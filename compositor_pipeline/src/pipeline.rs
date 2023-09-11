use std::collections::{hash_map, HashMap};
use std::sync::Arc;
use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use compositor_common::error::ErrorStack;
use compositor_common::renderer_spec::{RendererId, RendererSpec};
use compositor_common::scene::{InputId, OutputId, SceneSpec};
use compositor_common::{Frame, Framerate};
use compositor_render::error::{
    InitRendererEngineError, RegisterRendererError, UnregisterRendererError,
};
use compositor_render::event_loop::EventLoop;
use compositor_render::registry::RegistryType;
use compositor_render::renderer::RendererOptions;
use compositor_render::WebRendererOptions;
use compositor_render::{renderer::scene::UpdateSceneError, Renderer};
use crossbeam_channel::unbounded;
use log::{error, warn};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationMilliSeconds};

use crate::error::{
    RegisterInputError, RegisterOutputError, UnregisterInputError, UnregisterOutputError,
};
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

impl<Input: PipelineInput, Output: PipelineOutput> Pipeline<Input, Output> {
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
        F: Fn(OutputIterator<'_, Output>) -> R,
    {
        let guard = self.outputs.lock();
        f(OutputIterator::new(guard.iter()))
    }
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

pub struct OutputIterator<'a, Output: PipelineOutput> {
    inner_iter: hash_map::Iter<'a, OutputId, Arc<Output>>,
}

impl<'a, Output: PipelineOutput> OutputIterator<'a, Output> {
    fn new(iter: hash_map::Iter<'a, OutputId, Arc<Output>>) -> Self {
        Self { inner_iter: iter }
    }
}

impl<'a, Output: PipelineOutput> Iterator for OutputIterator<'a, Output> {
    type Item = (&'a OutputId, &'a Output);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter.next().map(|(id, node)| (id, node.as_ref()))
    }
}
