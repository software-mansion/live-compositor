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
use compositor_render::scene::{AudioComposition, Component};
use compositor_render::web_renderer::WebRendererInitOptions;
use compositor_render::{error::UpdateSceneError, Renderer};
use compositor_render::{AudioChannels, AudioMixer, RendererOptions};
use compositor_render::{AudioSamplesSet, FrameSet, RegistryType};
use compositor_render::{EventLoop, Framerate, InputId, OutputId, RendererId, RendererSpec};
use crossbeam_channel::{unbounded, Receiver};
use log::{debug, error, warn};

use crate::error::{
    RegisterInputError, RegisterOutputError, UnregisterInputError, UnregisterOutputError,
};
use crate::queue::Queue;

use self::decoder::DecoderOptions;
use self::encoder::{Encoder, EncoderOptions};
use self::input::InputOptions;
use self::output::{Output, OutputOptions};

pub mod decoder;
pub mod encoder;
pub mod input;
pub mod output;
mod structs;

pub use self::structs::AudioCodec;
pub use self::structs::VideoCodec;
pub use crate::queue::AudioOptions;
pub use crate::queue::InputType;

pub struct Port(pub u16);

pub enum RequestedPort {
    Exact(u16),
    Range((u16, u16)),
}

pub struct RegisterInputOptions {
    pub input_id: InputId,
    pub input_options: InputOptions,
    pub input_type: InputType,
    pub decoder_options: DecoderOptions,
}

pub struct OutputVideoOpts {
    pub encoder_opts: EncoderOptions,
    pub initial: Component,
}

pub struct OutputAudioOpts {
    pub initial: AudioComposition,
    pub sample_rate: u32,
    pub channels: AudioChannels,
}

pub struct RegisterOutputOptions {
    pub output_id: OutputId,
    pub output_options: OutputOptions,
    pub video: Option<OutputVideoOpts>,
    pub audio: Option<OutputAudioOpts>,
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
}

#[derive(Debug, Clone)]
pub struct Options {
    pub framerate: Framerate,
    pub stream_fallback_timeout: Duration,
    pub web_renderer: WebRendererInitOptions,
    pub force_gpu: bool,
    pub download_root: PathBuf,
}

impl Pipeline {
    pub fn new(opts: Options) -> Result<(Self, Arc<dyn EventLoop>), InitPipelineError> {
        let (renderer, event_loop) = Renderer::new(RendererOptions {
            web_renderer: opts.web_renderer,
            framerate: opts.framerate,
            stream_fallback_timeout: opts.stream_fallback_timeout,
            force_gpu: opts.force_gpu,
        })?;

        let mut download_dir = opts.download_root;
        download_dir.push(format!("live-compositor-{}", rand::random::<u64>()));
        std::fs::create_dir_all(&download_dir).map_err(InitPipelineError::CreateDownloadDir)?;

        let pipeline = Pipeline {
            outputs: OutputRegistry::new(),
            inputs: HashMap::new(),
            queue: Arc::new(Queue::new(opts.framerate)),
            renderer,
            audio_mixer: AudioMixer::new(),
            is_started: false,
            download_dir,
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
        register_options: RegisterInputOptions,
    ) -> Result<Option<Port>, RegisterInputError> {
        // TODO: the Option<Port> in the return type is a bit weird
        let RegisterInputOptions {
            input_id,
            input_options,
            input_type,
            decoder_options,
        } = register_options;

        if self.inputs.contains_key(&input_id) {
            return Err(RegisterInputError::AlreadyRegistered(input_id));
        }

        let (input, chunks_receiver, port) =
            input::Input::new(input_options, &self.download_dir)
                .map_err(|e| RegisterInputError::InputError(input_id.clone(), e))?;

        let decoder = decoder::Decoder::new(
            input_id.clone(),
            self.queue.clone(),
            chunks_receiver,
            decoder_options,
        )
        .map_err(|e| RegisterInputError::DecoderError(input_id.clone(), e))?;

        let pipeline_input = PipelineInput { input, decoder };

        self.inputs.insert(input_id.clone(), pipeline_input.into());

        self.queue.add_input(input_type);
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
            output_options,
            video,
            audio,
        } = register_options;
        let (has_video, has_audio) = (video.is_some(), audio.is_some());
        if !has_video && !has_audio {
            return Err(RegisterOutputError::NoVideoOrAudio(output_id));
        }

        // TODO handle only audio
        let Some(video_opts) = video else {
            return Ok(());
        };

        if self.outputs.contains_key(&output_id) {
            return Err(RegisterOutputError::AlreadyRegistered(output_id));
        }

        let EncoderOptions::H264(ref opts) = video_opts.encoder_opts;
        if opts.resolution.width % 2 != 0 || opts.resolution.height % 2 != 0 {
            return Err(RegisterOutputError::UnsupportedResolution(output_id));
        }

        let (encoder, packets) = Encoder::new(video_opts.encoder_opts)
            .map_err(|e| RegisterOutputError::EncoderError(output_id.clone(), e))?;

        let output = Output::new(output_options, packets)
            .map_err(|e| RegisterOutputError::OutputError(output_id.clone(), e))?;

        let output = PipelineOutput {
            encoder,
            output,
            has_video,
            has_audio,
        };

        self.outputs.insert(output_id.clone(), output.into());
        self.update_scene_root(output_id.clone(), video_opts.initial)
            .map_err(|e| RegisterOutputError::SceneError(output_id.clone(), e))?;

        if let Some(audio_opts) = audio {
            self.audio_mixer.register_output(
                output_id,
                audio_opts.sample_rate,
                audio_opts.channels,
                audio_opts.initial,
            );
        }

        Ok(())
    }

    pub fn unregister_output(&self, output_id: &OutputId) -> Result<(), UnregisterOutputError> {
        if !self.outputs.contains_key(output_id) {
            return Err(UnregisterOutputError::NotFound(output_id.clone()));
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

    pub fn update_scene(
        &mut self,
        output_id: OutputId,
        root_component: Option<Component>,
        audio_composition: Option<AudioComposition>,
    ) -> Result<(), UpdateSceneError> {
        self.check_output_spec(&output_id, &root_component, &audio_composition)?;
        if let Some(scene_root) = root_component {
            self.update_scene_root(output_id.clone(), scene_root)?;
        }

        if let Some(audio_composition) = audio_composition {
            self.update_audio_composition(output_id, audio_composition)?;
        }

        Ok(())
    }

    fn check_output_spec(
        &self,
        output_id: &OutputId,
        root_component: &Option<Component>,
        audio_composition: &Option<AudioComposition>,
    ) -> Result<(), UpdateSceneError> {
        let outputs = self.outputs.0.lock().unwrap();
        let Some(output) = outputs.get(output_id) else {
            return Err(UpdateSceneError::OutputNotRegistered(output_id.clone()));
        };
        if output.has_audio != audio_composition.is_some()
            || output.has_video != root_component.is_some()
        {
            return Err(UpdateSceneError::AudioVideoNotMatching(output_id.clone()));
        }
        if root_component.is_none() && audio_composition.is_none() {
            return Err(UpdateSceneError::NoAudioAndVideo(output_id.clone()));
        }
        Ok(())
    }

    fn update_scene_root(
        &mut self,
        output_id: OutputId,
        scene_root: Component,
    ) -> Result<(), UpdateSceneError> {
        let resolution = self
            .outputs
            .lock()
            .get(&output_id)
            .ok_or_else(|| UpdateSceneError::OutputNotRegistered(output_id.clone()))?
            .encoder
            .resolution();

        self.renderer
            .update_scene(output_id, resolution, scene_root)
    }

    fn update_audio_composition(
        &mut self,
        output_id: OutputId,
        audio_composition: AudioComposition,
    ) -> Result<(), UpdateSceneError> {
        self.audio_mixer.update_scene(output_id, audio_composition)
    }

    pub fn start(&mut self) {
        if self.is_started {
            error!("Pipeline already started.");
            return;
        }
        let (frames_sender, frames_receiver) = unbounded();
        let (audio_sender, audio_receiver) = unbounded();
        let renderer = self.renderer.clone();
        let audio_mixer = self.audio_mixer.clone();
        let outputs = self.outputs.clone();

        self.queue.start(frames_sender, audio_sender);

        thread::spawn(move || Self::run_renderer_thread(frames_receiver, renderer, outputs));
        thread::spawn(move || Self::run_audio_mixer_thread(audio_mixer, audio_receiver));
    }

    fn run_renderer_thread(
        frames_receiver: Receiver<FrameSet<InputId>>,
        renderer: Renderer,
        outputs: OutputRegistry<PipelineOutput>,
    ) {
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
                let output = outputs.lock().get(&id).cloned();
                let Some(output) = output else {
                    error!("no output with id {}", &id);
                    continue;
                };

                output.encoder.send_frame(frame);
            }
        }
    }

    fn run_audio_mixer_thread(audio_mixer: AudioMixer, audio_receiver: Receiver<AudioSamplesSet>) {
        for samples in audio_receiver {
            let mixed_samples = audio_mixer.mix_samples(samples.clone());
            debug!("Mixed samples: {:#?}", mixed_samples);
            // TODO send to output
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
