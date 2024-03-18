use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use compositor_render::error::{
    ErrorStack, InitPipelineError, RegisterRendererError, UnregisterRendererError,
};
use compositor_render::scene::Component;
use compositor_render::web_renderer::WebRendererInitOptions;
use compositor_render::RegistryType;
use compositor_render::RendererOptions;
use compositor_render::{error::UpdateSceneError, Renderer};
use compositor_render::{EventLoop, InputId, OutputId, RendererId, RendererSpec};
use crossbeam_channel::{bounded, Receiver};
use tracing::{error, info, warn};

use crate::audio_mixer::AudioMixer;
use crate::audio_mixer::{AudioChannels, AudioMixingParams};
use crate::error::{
    RegisterInputError, RegisterOutputError, UnregisterInputError, UnregisterOutputError,
};
use crate::queue::PipelineEvent;
use crate::queue::QueueAudioOutput;
use crate::queue::{self, Queue, QueueOptions, QueueVideoOutput};

use self::encoder::{AudioEncoderPreset, VideoEncoderOptions};
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
    pub encoder_preset: AudioEncoderPreset,
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

    /// Some(received) - Whether EOS was received from queue on audio stream for that input.
    /// None - No audio configured for that input.
    audio_eos_received: Option<bool>,
    /// Some(received) - Whether EOS was received from queue on video stream for that input.
    /// None - No video configured for that input.
    video_eos_received: Option<bool>,
}

pub struct PipelineOutput {
    pub encoder: encoder::Encoder,
    pub output: output::Output,
    pub has_video: bool,
    pub has_audio: bool,
}

pub struct Pipeline {
    inputs: HashMap<InputId, PipelineInput>,
    outputs: HashMap<OutputId, PipelineOutput>,
    queue: Arc<Queue>,
    renderer: Renderer,
    audio_mixer: AudioMixer,
    is_started: bool,
    download_dir: PathBuf,
    output_sample_rate: u32,
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
            outputs: HashMap::new(),
            inputs: HashMap::new(),
            queue: Queue::new(opts.queue_options),
            renderer,
            audio_mixer: AudioMixer::new(opts.output_sample_rate),
            is_started: false,
            download_dir,
            output_sample_rate: opts.output_sample_rate,
        };

        Ok((pipeline, event_loop))
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

        let (pipeline_input, receiver, port) = new_pipeline_input(
            &input_id,
            input_options,
            &self.download_dir,
            self.output_sample_rate,
        )?;

        self.audio_mixer.register_input(input_id.clone());
        self.inputs.insert(input_id.clone(), pipeline_input);
        self.queue.add_input(&input_id, receiver, queue_options);
        self.renderer.register_input(input_id);
        Ok(port)
    }

    pub fn unregister_input(&mut self, input_id: &InputId) -> Result<(), UnregisterInputError> {
        if !self.inputs.contains_key(input_id) {
            return Err(UnregisterInputError::NotFound(input_id.clone()));
        }

        self.audio_mixer.unregister_input(input_id);
        self.inputs.remove(input_id);
        self.queue.remove_input(input_id);
        self.renderer.unregister_input(input_id);
        Ok(())
    }

    pub fn register_output(
        &mut self,
        register_options: RegisterOutputOptions,
    ) -> Result<Option<Port>, RegisterOutputError> {
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

        let (output, port) = new_pipeline_output(register_options, self.output_sample_rate)?;
        self.outputs.insert(output_id.clone(), output);

        if let Some(audio_opts) = audio.clone() {
            self.audio_mixer.register_output(
                output_id.clone(),
                audio_opts.initial,
                audio_opts.channels,
            );
        }

        self.update_output(
            output_id.clone(),
            video.map(|v| v.initial),
            audio.map(|a| a.initial),
        )
        .map_err(|e| RegisterOutputError::SceneError(output_id, e))?;

        Ok(port)
    }

    pub fn unregister_output(&mut self, output_id: &OutputId) -> Result<(), UnregisterOutputError> {
        if !self.outputs.contains_key(output_id) {
            return Err(UnregisterOutputError::NotFound(output_id.clone()));
        }

        self.audio_mixer.unregister_output(output_id);
        self.outputs.remove(output_id);
        self.renderer.unregister_output(output_id);
        Ok(())
    }

    pub fn register_renderer(
        pipeline: &Arc<Mutex<Self>>,
        transformation_spec: RendererSpec,
    ) -> Result<(), RegisterRendererError> {
        let renderer = pipeline.lock().unwrap().renderer.clone();
        renderer.register_renderer(transformation_spec)?;
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
        let Some(output) = self.outputs.get(output_id) else {
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
            .get(&output_id)
            .ok_or_else(|| UpdateSceneError::OutputNotRegistered(output_id.clone()))?
            .encoder
            .video
            .as_ref()
            .map(|v| v.resolution())
        else {
            return Err(UpdateSceneError::AudioVideoNotMatching(output_id));
        };

        info!("Update scene {:#?}", scene_root);

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

    pub fn start(pipeline: &Arc<Mutex<Self>>) {
        let guard = pipeline.lock().unwrap();
        if guard.is_started {
            error!("Pipeline already started.");
            return;
        }
        info!("Starting pipeline.");
        let (video_sender, video_receiver) = bounded(1);
        let (audio_sender, audio_receiver) = bounded(100);
        guard.queue.start(video_sender, audio_sender);

        let pipeline_clone = pipeline.clone();
        thread::spawn(move || run_renderer_thread(pipeline_clone, video_receiver));

        let pipeline_clone = pipeline.clone();
        thread::spawn(move || run_audio_mixer_thread(pipeline_clone, audio_receiver));
    }

    pub fn inputs(&self) -> impl Iterator<Item = (&InputId, &PipelineInput)> {
        self.inputs.iter()
    }

    pub fn outputs(&self) -> impl Iterator<Item = (&OutputId, &PipelineOutput)> {
        self.outputs.iter()
    }
}

fn run_renderer_thread(
    pipeline: Arc<Mutex<Pipeline>>,
    frames_receiver: Receiver<QueueVideoOutput>,
) {
    let renderer = pipeline.lock().unwrap().renderer.clone();
    for mut input_frames in frames_receiver.iter() {
        for (input_id, event) in input_frames.frames.iter_mut() {
            if let PipelineEvent::EOS = event {
                let mut guard = pipeline.lock().unwrap();
                let eos_received = guard
                    .inputs
                    .get_mut(input_id)
                    .and_then(|input| input.video_eos_received.as_mut());
                if let Some(eos_received) = eos_received {
                    *eos_received = true;
                }
            }
        }
        let output = renderer.render(input_frames.into());
        let Ok(output_frames) = output else {
            error!(
                "Error while rendering: {}",
                ErrorStack::new(&output.unwrap_err()).into_string()
            );
            continue;
        };

        for (id, frame) in output_frames.frames {
            let frame_sender = pipeline
                .lock()
                .unwrap()
                .outputs
                .get(&id)
                .and_then(|output| output.encoder.frame_sender())
                .cloned();
            let Some(frame_sender) = frame_sender else {
                warn!("Failed to send output frame. No output with id {}.", &id);
                continue;
            };

            if frame_sender.send(PipelineEvent::Data(frame)).is_err() {
                warn!("Failed to send output frames. Channel closed.")
            }
        }
    }
}

fn run_audio_mixer_thread(
    pipeline: Arc<Mutex<Pipeline>>,
    audio_receiver: Receiver<QueueAudioOutput>,
) {
    let audio_mixer = pipeline.lock().unwrap().audio_mixer.clone();
    for mut samples in audio_receiver.iter() {
        for (input_id, event) in samples.samples.iter_mut() {
            if let PipelineEvent::EOS = event {
                let mut guard = pipeline.lock().unwrap();
                let eos_received = guard
                    .inputs
                    .get_mut(input_id)
                    .and_then(|input| input.audio_eos_received.as_mut());
                if let Some(eos_received) = eos_received {
                    *eos_received = true;
                }
            }
        }
        let mixed_samples = audio_mixer.mix_samples(samples.into());
        for (id, batch) in mixed_samples.0 {
            let samples_sender = pipeline
                .lock()
                .unwrap()
                .outputs
                .get(&id)
                .and_then(|output| output.encoder.samples_batch_sender())
                .cloned();
            let Some(samples_sender) = samples_sender else {
                warn!("Filed to send mixed audio. No output with id {}.", &id);
                continue;
            };

            if samples_sender.send(PipelineEvent::Data(batch)).is_err() {
                warn!("Failed to send mixed audio. Channel closed.")
            }
        }
    }
}
