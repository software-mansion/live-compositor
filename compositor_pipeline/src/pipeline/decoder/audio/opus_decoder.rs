use std::sync::Arc;

use crate::{
    error::DecoderInitError,
    pipeline::{
        decoder::{DecodedAudioFormat, OpusDecoderOptions},
        structs::EncodedChunk,
    },
};

use super::{AudioDecoderT, DecodedSamples, DecodingError};

pub(in super::super) struct OpusDecoder {
    decoder: opus::Decoder,
    decoded_samples_buffer: [i16; 100_000],
    forward_error_correction: bool,
    output_sample_rate: u32,
}

impl OpusDecoder {
    pub fn new(
        opts: OpusDecoderOptions,
        output_sample_rate: u32,
    ) -> Result<Self, DecoderInitError> {
        let decoder = opus::Decoder::new(output_sample_rate, opus::Channels::Stereo)?;
        // Max sample rate for opus is 48kHz.
        // Usually packets contain 20ms audio chunks, but for safety we use buffer
        // that can hold >1s of 48kHz stereo audio (96k samples)
        let decoded_samples_buffer = [0i16; 100_000];

        Ok(Self {
            decoder,
            decoded_samples_buffer,
            forward_error_correction: opts.forward_error_correction,
            output_sample_rate,
        })
    }

    /// Panics if buffer.len() < 2 * decoded_samples_count
    fn read_buffer(buffer: &[i16], decoded_samples_count: usize) -> Arc<Vec<(i16, i16)>> {
        Arc::new(
            buffer[0..(2 * decoded_samples_count)]
                .chunks_exact(2)
                .map(|c| (c[0], c[1]))
                .collect(),
        )
    }
}

impl AudioDecoderT for OpusDecoder {
    fn decode(
        &mut self,
        encoded_chunk: EncodedChunk,
    ) -> Result<Vec<DecodedSamples>, DecodingError> {
        let decoded_samples_count = self.decoder.decode(
            &encoded_chunk.data,
            &mut self.decoded_samples_buffer,
            self.forward_error_correction,
        )?;

        let samples = Self::read_buffer(&self.decoded_samples_buffer, decoded_samples_count);
        let decoded_samples = DecodedSamples {
            samples,
            start_pts: encoded_chunk.pts,
            sample_rate: self.output_sample_rate,
        };
        Ok(Vec::from([decoded_samples]))
    }

    fn decoded_format(&self) -> DecodedAudioFormat {
        DecodedAudioFormat {
            sample_rate: self.output_sample_rate,
        }
    }
}
