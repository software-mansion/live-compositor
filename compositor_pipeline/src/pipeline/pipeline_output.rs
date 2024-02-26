use crate::error::RegisterOutputError;

use super::{
    encoder::{opus_encoder, AudioEncoderOptions, AudioEncoderPreset, Encoder, EncoderOptions},
    output::Output,
    PipelineOutput, Port, RegisterOutputOptions,
};

pub(super) fn new_pipeline_output(
    opts: RegisterOutputOptions,
    output_sample_rate: u32,
) -> Result<(PipelineOutput, Option<Port>), RegisterOutputError> {
    let RegisterOutputOptions {
        output_id,
        output_options,
        video,
        audio,
    } = opts;
    let (has_video, has_audio) = (video.is_some(), audio.is_some());

    let encoder_opts = EncoderOptions {
        video: video.map(|video_opts| video_opts.encoder_opts),
        audio: audio.map(|audio_opts| {
            AudioEncoderOptions::Opus(opus_encoder::Options {
                sample_rate: output_sample_rate,
                channels: audio_opts.channels,
                preset: AudioEncoderPreset::Quality,
            })
        }),
    };

    let (encoder, packets) = Encoder::new(encoder_opts)
        .map_err(|e| RegisterOutputError::EncoderError(output_id.clone(), e))?;

    let (output, port) = Output::new(output_options, packets)
        .map_err(|e| RegisterOutputError::OutputError(output_id.clone(), e))?;

    let output = PipelineOutput {
        encoder,
        output,
        has_video,
        has_audio,
    };

    Ok((output, port))
}
