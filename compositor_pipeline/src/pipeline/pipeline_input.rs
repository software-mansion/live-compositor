use std::path::Path;

use crate::error::RegisterInputError;

use super::{
    decoder::{self, DecodedDataReceiver},
    input, PipelineInput, Port, RegisterInputOptions,
};

pub(super) fn new_pipeline_input(
    opts: RegisterInputOptions,
    download_dir: &Path,
) -> Result<(PipelineInput, DecodedDataReceiver, Option<Port>), RegisterInputError> {
    let RegisterInputOptions {
        input_id,
        input_options,
        decoder_options,
        ..
    } = opts;

    let (input, chunks_receiver, port) = input::Input::new(input_options, download_dir)
        .map_err(|e| RegisterInputError::InputError(input_id.clone(), e))?;

    let (decoder, decoded_data_receiver) =
        decoder::Decoder::new(input_id.clone(), chunks_receiver, decoder_options)
            .map_err(|e| RegisterInputError::DecoderError(input_id.clone(), e))?;

    let pipeline_input = PipelineInput { input, decoder };
    Ok((pipeline_input, decoded_data_receiver, port))
}
