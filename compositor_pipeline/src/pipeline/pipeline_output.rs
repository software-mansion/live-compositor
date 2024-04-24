use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use compositor_render::{Frame, InputId, OutputId};
use crossbeam_channel::Sender;
use tracing::{info, warn};

use crate::{audio_mixer::OutputSamples, error::RegisterOutputError, queue::PipelineEvent};

use super::{
    encoder::{self, opus, AudioEncoderOptions, Encoder, EncoderOptions},
    output::{self, Output},
    Pipeline, PipelineInput, Port, RegisterOutputOptions,
};

#[derive(Debug, Clone)]
pub enum PipelineOutputEndCondition {
    AnyOf(Vec<InputId>),
    AllOf(Vec<InputId>),
    AnyInput,
    AllInputs,
    Never,
}

pub struct PipelineOutput {
    pub encoder: encoder::Encoder,
    pub output: output::Output,
    pub video_end_condition: Option<PipelineOutputEndConditionState>,
    pub audio_end_condition: Option<PipelineOutputEndConditionState>,
}

impl Pipeline {
    pub(super) fn register_pipeline_output(
        &mut self,
        output_id: OutputId,
        opts: RegisterOutputOptions,
    ) -> Result<Option<Port>, RegisterOutputError> {
        let RegisterOutputOptions {
            video,
            audio,
            output_options,
        } = opts;
        let (has_video, has_audio) = (video.is_some(), audio.is_some());
        if !has_video && !has_audio {
            return Err(RegisterOutputError::NoVideoAndAudio(output_id));
        }

        if self.outputs.contains_key(&output_id) {
            return Err(RegisterOutputError::AlreadyRegistered(output_id));
        }

        let encoder_opts = EncoderOptions {
            video: video
                .as_ref()
                .map(|video_opts| video_opts.encoder_opts.clone()),
            audio: audio.as_ref().map(|audio_opts| {
                AudioEncoderOptions::Opus(opus::Options {
                    channels: audio_opts.channels,
                    preset: audio_opts.encoder_preset,
                })
            }),
        };

        let (encoder, packets) = Encoder::new(&output_id, encoder_opts, self.output_sample_rate)
            .map_err(|e| RegisterOutputError::EncoderError(output_id.clone(), e))?;

        let (output, port) = Output::new(&output_id, output_options, packets)
            .map_err(|e| RegisterOutputError::OutputError(output_id.clone(), e))?;

        let output = PipelineOutput {
            encoder,
            output,
            audio_end_condition: audio.as_ref().map(|audio| {
                PipelineOutputEndConditionState::new_audio(
                    audio.end_condition.clone(),
                    &self.inputs,
                )
            }),
            video_end_condition: video.as_ref().map(|video| {
                PipelineOutputEndConditionState::new_video(
                    video.end_condition.clone(),
                    &self.inputs,
                )
            }),
        };

        if let Some(video_opts) = video.clone() {
            let result = self.renderer.update_scene(
                output_id.clone(),
                video_opts.encoder_opts.resolution(),
                video_opts.initial,
            );

            if let Err(err) = result {
                self.renderer.unregister_output(&output_id);
                return Err(RegisterOutputError::SceneError(output_id.clone(), err));
            }
        };

        if let Some(audio_opts) = audio.clone() {
            self.audio_mixer.register_output(
                output_id.clone(),
                audio_opts.initial,
                audio_opts.mixing_strategy,
                audio_opts.channels,
            );
        }

        self.outputs.insert(output_id.clone(), output);

        Ok(port)
    }

    pub(super) fn get_output_video_sender(
        pipeline: &Arc<Mutex<Pipeline>>,
        output_id: &OutputId,
    ) -> Option<Sender<PipelineEvent<Frame>>> {
        let mut guard = pipeline.lock().unwrap();
        let Some(output) = guard.outputs.get_mut(output_id) else {
            warn!(
                ?output_id,
                "Failed to send output frame. Output does not exists.",
            );
            return None;
        };
        let sender = output.encoder.frame_sender()?.clone();
        let eos_status = output.video_end_condition.as_mut()?.eos_status();
        drop(guard);

        match eos_status {
            EosStatus::None => Some(sender),
            EosStatus::SendEos => {
                info!(?output_id, "Sending video EOS on output.");
                if sender.send(PipelineEvent::EOS).is_err() {
                    warn!(
                        ?output_id,
                        "Failed to send EOS from renderer. Channel closed."
                    );
                };
                None
            }
            EosStatus::AlreadySent => {
                warn!(?output_id, "Received new frame from renderer after EOS.");
                None
            }
        }
    }

    pub(super) fn get_output_audio_sender(
        pipeline: &Arc<Mutex<Pipeline>>,
        output_id: &OutputId,
    ) -> Option<Sender<PipelineEvent<OutputSamples>>> {
        let mut guard = pipeline.lock().unwrap();
        let Some(output) = guard.outputs.get_mut(output_id) else {
            warn!(
                ?output_id,
                "Failed to send output samples. Output does not exists.",
            );
            return None;
        };
        let sender = output.encoder.samples_batch_sender()?.clone();
        let eos_status = output.audio_end_condition.as_mut()?.eos_status();
        drop(guard);

        match eos_status {
            EosStatus::None => Some(sender),
            EosStatus::SendEos => {
                info!(?output_id, "Sending audio EOS on output.");
                if sender.send(PipelineEvent::EOS).is_err() {
                    warn!(?output_id, "Failed to send EOS from mixer. Channel closed.");
                };
                None
            }
            EosStatus::AlreadySent => {
                warn!(?output_id, "Received new mixed samples after EOS.");
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineOutputEndConditionState {
    condition: PipelineOutputEndCondition,
    connected_inputs: HashSet<InputId>,
    did_end: bool,
    did_send_eos: bool,
}

enum StateChange<'a> {
    AddInput(&'a InputId),
    RemoveInput(&'a InputId),
    NoChanges,
}

enum EosStatus {
    None,
    SendEos,
    AlreadySent,
}

impl PipelineOutputEndConditionState {
    fn new_video(
        condition: PipelineOutputEndCondition,
        inputs: &HashMap<InputId, PipelineInput>,
    ) -> Self {
        Self {
            condition,
            connected_inputs: inputs
                .iter()
                .filter_map(|(input_id, input)| match input.video_eos_received {
                    Some(false) => Some(input_id.clone()),
                    _ => None,
                })
                .collect(),
            did_end: false,
            did_send_eos: false,
        }
    }

    fn new_audio(
        condition: PipelineOutputEndCondition,
        inputs: &HashMap<InputId, PipelineInput>,
    ) -> Self {
        Self {
            condition,
            connected_inputs: inputs
                .iter()
                .filter_map(|(input_id, input)| match input.audio_eos_received {
                    Some(false) => Some(input_id.clone()),
                    _ => None,
                })
                .collect(),
            did_end: false,
            did_send_eos: false,
        }
    }

    fn eos_status(&mut self) -> EosStatus {
        self.on_event(StateChange::NoChanges);
        if self.did_end {
            if !self.did_send_eos {
                self.did_send_eos = true;
                return EosStatus::SendEos;
            }
            return EosStatus::AlreadySent;
        }
        EosStatus::None
    }

    pub(super) fn on_input_registered(&mut self, input_id: &InputId) {
        self.on_event(StateChange::AddInput(input_id))
    }
    pub(super) fn on_input_unregistered(&mut self, input_id: &InputId) {
        self.on_event(StateChange::RemoveInput(input_id))
    }
    pub(super) fn on_input_eos(&mut self, input_id: &InputId) {
        self.on_event(StateChange::RemoveInput(input_id))
    }

    fn on_event(&mut self, action: StateChange) {
        if self.did_end {
            return;
        }
        match action {
            StateChange::AddInput(id) => {
                self.connected_inputs.insert(id.clone());
            }
            StateChange::RemoveInput(id) => {
                self.connected_inputs.remove(id);
            }
            StateChange::NoChanges => (),
        };
        self.did_end = match self.condition {
            PipelineOutputEndCondition::AnyOf(ref inputs) => inputs
                .iter()
                .any(|input_id| !self.connected_inputs.contains(input_id)),
            PipelineOutputEndCondition::AllOf(ref inputs) => inputs
                .iter()
                .all(|input_id| !self.connected_inputs.contains(input_id)),
            PipelineOutputEndCondition::AnyInput => matches!(action, StateChange::RemoveInput(_)),
            PipelineOutputEndCondition::AllInputs => self.connected_inputs.is_empty(),
            PipelineOutputEndCondition::Never => false,
        };
    }
}
