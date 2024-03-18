use compositor_render::InputId;

use crate::{error::RegisterInputError, Pipeline};

use super::{decoder, input, Port, RegisterInputOptions};

pub struct PipelineInput {
    pub input: input::Input,
    pub decoder: decoder::Decoder,

    /// Some(received) - Whether EOS was received from queue on audio stream for that input.
    /// None - No audio configured for that input.
    pub(super) audio_eos_received: Option<bool>,
    /// Some(received) - Whether EOS was received from queue on video stream for that input.
    /// None - No video configured for that input.
    pub(super) video_eos_received: Option<bool>,
}

impl Pipeline {
    pub(super) fn register_pipeline_input(
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

        let (input, chunks_receiver, decoder_options, port) =
            input::Input::new(input_options, &self.download_dir)
                .map_err(|e| RegisterInputError::InputError(input_id.clone(), e))?;

        let (audio_eos_received, video_eos_received) = (
            decoder_options.audio.as_ref().map(|_| false),
            decoder_options.video.as_ref().map(|_| false),
        );
        let (decoder, decoded_data_receiver) = decoder::Decoder::new(
            input_id.clone(),
            chunks_receiver,
            decoder_options,
            self.output_sample_rate,
        )
        .map_err(|e| RegisterInputError::DecoderError(input_id.clone(), e))?;

        let pipeline_input = PipelineInput {
            input,
            decoder,
            audio_eos_received,
            video_eos_received,
        };

        if pipeline_input.audio_eos_received.is_some() {
            for (_, output) in self.outputs.iter_mut() {
                if let Some(ref mut cond) = output.audio_end_condition {
                    cond.on_input_registered(&input_id);
                }
            }
        }

        if pipeline_input.video_eos_received.is_some() {
            for (_, output) in self.outputs.iter_mut() {
                if let Some(ref mut cond) = output.video_end_condition {
                    cond.on_input_registered(&input_id);
                }
            }
        }

        self.inputs.insert(input_id.clone(), pipeline_input);
        self.queue
            .add_input(&input_id, decoded_data_receiver, queue_options);
        self.renderer.register_input(input_id);

        Ok(port)
    }
}

impl PipelineInput {
    pub(super) fn on_audio_eos(&mut self) {
        self.audio_eos_received = self.audio_eos_received.map(|_| true);
    }
    pub(super) fn on_video_eos(&mut self) {
        self.audio_eos_received = self.audio_eos_received.map(|_| true);
    }
}
