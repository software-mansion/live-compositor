use compositor_render::OutputId;

use crate::error::RegisterOutputError;

use super::{
    encoder::{Encoder, EncoderOptions},
    output::Output,
    PipelineOutput, RegisterOutputOptions,
};

pub(super) fn new_pipeline_output(
    output_id: &OutputId,
    opts: RegisterOutputOptions,
) -> Result<PipelineOutput, RegisterOutputError> {
    let RegisterOutputOptions {
        encoder_options,
        output_options,
    } = opts;
    let EncoderOptions::H264(ref h264_opts) = encoder_options;
    if h264_opts.resolution.width % 2 != 0 || h264_opts.resolution.height % 2 != 0 {
        return Err(RegisterOutputError::UnsupportedResolution(
            output_id.clone(),
        ));
    }

    let (encoder, packets) = Encoder::new(encoder_options)
        .map_err(|e| RegisterOutputError::EncoderError(output_id.clone(), e))?;

    let output = Output::new(output_options, packets)
        .map_err(|e| RegisterOutputError::OutputError(output_id.clone(), e))?;

    let output = PipelineOutput { encoder, output };

    Ok(output)
}
