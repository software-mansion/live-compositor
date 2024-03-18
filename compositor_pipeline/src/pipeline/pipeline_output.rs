use std::collections::{HashMap, HashSet};

use compositor_render::InputId;

use crate::error::RegisterOutputError;

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
        opts: RegisterOutputOptions,
    ) -> Result<Option<Port>, RegisterOutputError> {
        let RegisterOutputOptions {
            output_id,
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

        let (encoder, packets) = Encoder::new(encoder_opts, self.output_sample_rate)
            .map_err(|e| RegisterOutputError::EncoderError(output_id.clone(), e))?;

        let (output, port) = Output::new(output_options, packets)
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
                audio_opts.channels,
            );
        }

        self.outputs.insert(output_id.clone(), output);

        Ok(port)
    }
}

#[derive(Debug, Clone)]
pub struct PipelineOutputEndConditionState {
    condition: PipelineOutputEndCondition,
    connected_inputs: HashSet<InputId>,
    did_end: bool,
    did_send_eos: bool,
}

enum InputAction<'a> {
    AddInput(&'a InputId),
    RemoveInput(&'a InputId),
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

    pub(super) fn should_send_eos(&mut self) -> bool {
        if self.did_end && !self.did_send_eos {
            self.did_send_eos = true;
            return true;
        }
        false
    }

    pub(super) fn on_input_registered(&mut self, input_id: &InputId) {
        self.on_event(InputAction::AddInput(input_id))
    }
    pub(super) fn on_input_unregistered(&mut self, input_id: &InputId) {
        self.on_event(InputAction::RemoveInput(input_id))
    }
    pub(super) fn on_input_eos(&mut self, input_id: &InputId) {
        self.on_event(InputAction::RemoveInput(input_id))
    }

    fn on_event(&mut self, action: InputAction) {
        match action {
            InputAction::AddInput(id) => self.connected_inputs.insert(id.clone()),
            InputAction::RemoveInput(id) => self.connected_inputs.remove(id),
        };
        self.did_end = match self.condition {
            PipelineOutputEndCondition::AnyOf(ref inputs) => {
                let connected_inputs_from_list = inputs
                    .iter()
                    .filter(|input_id| self.connected_inputs.get(input_id).is_some())
                    .count();
                connected_inputs_from_list != inputs.len()
            }
            PipelineOutputEndCondition::AllOf(ref inputs) => {
                let connected_inputs_from_list = inputs
                    .iter()
                    .filter(|input_id| self.connected_inputs.get(input_id).is_some())
                    .count();
                connected_inputs_from_list == 0
            }
            PipelineOutputEndCondition::AnyInput => matches!(action, InputAction::RemoveInput(_)),
            PipelineOutputEndCondition::AllInputs => self.connected_inputs.is_empty(),
            PipelineOutputEndCondition::Never => false,
        };
    }
}
