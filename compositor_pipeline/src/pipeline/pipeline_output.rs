use crate::error::RegisterOutputError;

use super::{
    encoder::{VideoEncoder, VideoEncoderOptions},
    output::Output,
    PipelineOutput, RegisterOutputOptions,
};

pub(super) fn new_pipeline_output(
    opts: RegisterOutputOptions,
) -> Result<PipelineOutput, RegisterOutputError> {
    let RegisterOutputOptions {
        output_id,
        output_options,
        video,
        audio,
    } = opts;
    let encoder_opts = video.clone().unwrap().encoder_opts;
    let VideoEncoderOptions::H264(ref h264_opts) = encoder_opts;
    if h264_opts.resolution.width % 2 != 0 || h264_opts.resolution.height % 2 != 0 {
        return Err(RegisterOutputError::UnsupportedResolution(
            output_id.clone(),
        ));
    }

    let (encoder, packets) = VideoEncoder::new(encoder_opts)
        .map_err(|e| RegisterOutputError::EncoderError(output_id.clone(), e))?;

    let output = Output::new(output_options, packets)
        .map_err(|e| RegisterOutputError::OutputError(output_id.clone(), e))?;

    let output = PipelineOutput {
        encoder,
        output,
        has_video: video.is_some(),
        has_audio: audio.is_some(),
    };

    Ok(output)
}
