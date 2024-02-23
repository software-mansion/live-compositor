use std::collections::{hash_map, HashMap};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use compositor_render::error::{
    ErrorStack, InitPipelineError, RegisterRendererError, UnregisterRendererError,
};
use compositor_render::scene::Component;
use compositor_render::web_renderer::WebRendererInitOptions;
use compositor_render::RendererOptions;
use compositor_render::{error::UpdateSceneError, Renderer};
use compositor_render::{EventLoop, InputId, OutputId, RendererId, RendererSpec};
use compositor_render::{FrameSet, RegistryType};
use crossbeam_channel::{bounded, Receiver};
use log::error;

use crate::audio_mixer::types::{AudioChannels, AudioMixingParams, AudioSamplesSet};
use crate::audio_mixer::AudioMixer;
use crate::error::{
    RegisterInputError, RegisterOutputError, UnregisterInputError, UnregisterOutputError,
};
use crate::queue::{self, Queue, QueueOptions};

use self::encoder::VideoEncoderOptions;
use self::input::InputOptions;
use self::output::OutputOptions;

pub mod decoder;
pub mod encoder;
pub mod input;
pub mod output;
mod pipeline_input;
mod pipeline_output;
pub mod rtp;
mod structs;

use self::pipeline_input::new_pipeline_input;
use self::pipeline_output::new_pipeline_output;
pub use self::structs::AudioCodec;
pub use self::structs::VideoCodec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port(pub u16);

pub struct RegisterInputOptions {
    pub input_options: InputOptions,
    pub queue_options: queue::InputOptions,
}

#[derive(Debug, Clone)]
pub struct OutputVideoOptions {
    pub encoder_opts: VideoEncoderOptions,
    pub initial: Component,
}

#[derive(Debug, Clone)]
pub struct OutputAudioOptions {
    pub initial: AudioMixingParams,
    pub channels: AudioChannels,
    pub forward_error_correction: bool,
}

#[derive(Debug, Clone)]
pub struct RegisterOutputOptions {
    pub output_id: OutputId,
    pub output_options: OutputOptions,
    pub video: Option<OutputVideoOptions>,
    pub audio: Option<OutputAudioOptions>,
}

#[derive(Debug, Clone)]
pub struct OutputScene {
    pub output_id: OutputId,
    pub scene_root: Component,
}

pub struct PipelineInput {
    pub input: input::Input,
    pub decoder: decoder::Decoder,
}

pub struct PipelineOutput {
    pub encoder: encoder::Encoder,
    pub output: output::Output,
    pub has_video: bool,
    pub has_audio: bool,
}

pub struct Pipeline {
    inputs: HashMap<InputId, Arc<PipelineInput>>,
    outputs: OutputRegistry<PipelineOutput>,
    queue: Arc<Queue>,
    renderer: Renderer,
    audio_mixer: AudioMixer,
    is_started: bool,
    download_dir: PathBuf,
    pub output_sample_rate: u32,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub queue_options: QueueOptions,
    pub stream_fallback_timeout: Duration,
    pub web_renderer: WebRendererInitOptions,
    pub force_gpu: bool,
    pub download_root: PathBuf,
    pub output_sample_rate: u32,
}

impl Pipeline {
    pub fn new(opts: Options) -> Result<(Self, Arc<dyn EventLoop>), InitPipelineError> {
        let (renderer, event_loop) = Renderer::new(RendererOptions {
            web_renderer: opts.web_renderer,
            framerate: opts.queue_options.output_framerate,
            stream_fallback_timeout: opts.stream_fallback_timeout,
            force_gpu: opts.force_gpu,
        })?;

        let mut download_dir = opts.download_root;
        download_dir.push(format!("live-compositor-{}", rand::random::<u64>()));
        std::fs::create_dir_all(&download_dir).map_err(InitPipelineError::CreateDownloadDir)?;

        let pipeline = Pipeline {
            outputs: OutputRegistry::new(),
            inputs: HashMap::new(),
            queue: Queue::new(opts.queue_options),
            renderer,
            audio_mixer: AudioMixer::new(),
            is_started: false,
            download_dir,
            output_sample_rate: opts.output_sample_rate,
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
        register_options: RegisterInputOptions,
    ) -> Result<Option<Port>, RegisterInputError> {
        let RegisterInputOptions {
            input_options,
            queue_options,
        } = register_options;
        if self.inputs.contains_key(&input_id) {
            return Err(RegisterInputError::AlreadyRegistered(input_id));
        }

        let (pipeline_input, receiver, port) =
            new_pipeline_input(&input_id, input_options, &self.download_dir)?;

        self.inputs.insert(input_id.clone(), pipeline_input.into());
        self.queue.add_input(&input_id, receiver, queue_options);
        self.renderer.register_input(input_id);
        Ok(port)
    }

    pub fn unregister_input(&mut self, input_id: &InputId) -> Result<(), UnregisterInputError> {
        if !self.inputs.contains_key(input_id) {
            return Err(UnregisterInputError::NotFound(input_id.clone()));
        }

        self.inputs.remove(input_id);
        self.queue.remove_input(input_id);
        self.renderer.unregister_input(input_id);
        Ok(())
    }

    pub fn register_output(
        &mut self,
        register_options: RegisterOutputOptions,
    ) -> Result<(), RegisterOutputError> {
        let RegisterOutputOptions {
            output_id,
            video,
            audio,
            ..
        } = register_options.clone();
        let (has_video, has_audio) = (video.is_some(), audio.is_some());
        if !has_video && !has_audio {
            return Err(RegisterOutputError::NoVideoAndAudio(output_id));
        }

        if self.outputs.contains_key(&output_id) {
            return Err(RegisterOutputError::AlreadyRegistered(output_id));
        }

        let output = new_pipeline_output(register_options, self.output_sample_rate)?;
        self.outputs.insert(output_id.clone(), Arc::new(output));

        if let Some(audio_opts) = audio.clone() {
            self.audio_mixer
                .register_output(output_id.clone(), audio_opts.initial);
        }

        self.update_output(
            output_id.clone(),
            video.map(|v| v.initial),
            audio.map(|a| a.initial),
        )
        .map_err(|e| RegisterOutputError::SceneError(output_id, e))?;

        Ok(())
    }

    pub fn unregister_output(&self, output_id: &OutputId) -> Result<(), UnregisterOutputError> {
        if !self.outputs.contains_key(output_id) {
            return Err(UnregisterOutputError::NotFound(output_id.clone()));
        }

        self.audio_mixer.unregister_output(output_id);
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

    pub fn update_output(
        &mut self,
        output_id: OutputId,
        root_component: Option<Component>,
        audio: Option<AudioMixingParams>,
    ) -> Result<(), UpdateSceneError> {
        self.check_output_spec(&output_id, &root_component, &audio)?;
        if let Some(root_component) = root_component {
            self.update_scene_root(output_id.clone(), root_component)?;
        }

        if let Some(audio) = audio {
            self.update_audio(&output_id, audio)?;
        }

        Ok(())
    }

    fn check_output_spec(
        &self,
        output_id: &OutputId,
        root_component: &Option<Component>,
        audio: &Option<AudioMixingParams>,
    ) -> Result<(), UpdateSceneError> {
        let outputs = self.outputs.0.lock().unwrap();
        let Some(output) = outputs.get(output_id) else {
            return Err(UpdateSceneError::OutputNotRegistered(output_id.clone()));
        };
        if output.has_audio != audio.is_some() || output.has_video != root_component.is_some() {
            return Err(UpdateSceneError::AudioVideoNotMatching(output_id.clone()));
        }
        if root_component.is_none() && audio.is_none() {
            return Err(UpdateSceneError::NoAudioAndVideo(output_id.clone()));
        }
        Ok(())
    }

    fn update_scene_root(
        &mut self,
        output_id: OutputId,
        scene_root: Component,
    ) -> Result<(), UpdateSceneError> {
        let Some(resolution) = self
                    .outputs
                    .lock()
                    .get(&output_id)
                    .ok_or_else(|| UpdateSceneError::OutputNotRegistered(output_id.clone()))?
                    .encoder
                    .video
                    .as_ref()
                    .map(|v| v.resolution())
            else {
                return Err(UpdateSceneError::AudioVideoNotMatching(output_id));
            };

        self.renderer
            .update_scene(output_id, resolution, scene_root)
    }

    fn update_audio(
        &mut self,
        output_id: &OutputId,
        audio: AudioMixingParams,
    ) -> Result<(), UpdateSceneError> {
        self.audio_mixer.update_output(output_id, audio)
    }

    pub fn start(&mut self) {
        if self.is_started {
            error!("Pipeline already started.");
            return;
        }
        let (frames_sender, frames_receiver) = bounded(20);
        // for 20ms chunks this will be 60 seconds of audio
        let (audio_sender, audio_receiver) = bounded(300);
        let renderer = self.renderer.clone();
        let audio_mixer = self.audio_mixer.clone();
        let outputs = self.outputs.clone();
        let outputs2 = self.outputs.clone();

        self.queue.start(frames_sender, audio_sender);

        thread::spawn(move || Self::run_renderer_thread(frames_receiver, renderer, outputs));
        thread::spawn(move || Self::run_audio_mixer_thread(audio_mixer, audio_receiver, outputs2));
    }

    fn run_renderer_thread(
        frames_receiver: Receiver<FrameSet<InputId>>,
        renderer: Renderer,
        outputs: OutputRegistry<PipelineOutput>,
    ) {
        for input_frames in frames_receiver.iter() {
            let output = renderer.render(input_frames);
            let Ok(output_frames) = output else {
                error!(
                    "Error while rendering: {}",
                    ErrorStack::new(&output.unwrap_err()).into_string()
                );
                continue;
            };

            for (id, frame) in output_frames.frames {
                let output = outputs.lock().get(&id).cloned();
                let Some(output) = output else {
                    error!("no output with id {}", &id);
                    continue;
                };

                output.encoder.send_frame(frame);
            }
        }
    }

    fn run_audio_mixer_thread(
        audio_mixer: AudioMixer,
        audio_receiver: Receiver<AudioSamplesSet>,
        outputs: OutputRegistry<PipelineOutput>,
    ) {
        for samples in audio_receiver {
            let mixed_samples = audio_mixer.mix_samples(samples.clone());
            for (id, batch) in mixed_samples.0 {
                let output = outputs.lock().get(&id).cloned();
                let Some(output) = output else {
                    error!("no output with id {}", &id);
                    continue;
                };

                output.encoder.send_samples_batch(batch);
            }
        }
    }

    pub fn inputs(&self) -> impl Iterator<Item = (&InputId, &PipelineInput)> {
        self.inputs.iter().map(|(id, node)| (id, node.deref()))
    }

    pub fn with_outputs<F, R>(&self, f: F) -> R
    where
        F: Fn(OutputIterator<'_>) -> R,
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

pub struct OutputIterator<'a> {
    inner_iter: hash_map::Iter<'a, OutputId, Arc<PipelineOutput>>,
}

impl<'a> OutputIterator<'a> {
    fn new(iter: hash_map::Iter<'a, OutputId, Arc<PipelineOutput>>) -> Self {
        Self { inner_iter: iter }
    }
}

impl<'a> Iterator for OutputIterator<'a> {
    type Item = (&'a OutputId, &'a PipelineOutput);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter.next().map(|(id, node)| (id, node.deref()))
    }
}
