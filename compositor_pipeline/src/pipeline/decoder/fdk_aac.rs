use compositor_render::InputId;
use crossbeam_channel::{Receiver, Sender};
use fdk_aac_sys as fdk;
use tracing::{debug, error, span, Level};

use crate::{
    audio_mixer::types::{AudioSamples, AudioSamplesBatch},
    pipeline::structs::{EncodedChunk, EncodedChunkKind},
    queue::PipelineEvent,
};

use super::AacDecoderOptions;

#[derive(Debug, thiserror::Error)]
pub enum AacDecoderError {
    #[error("The internal fdk decoder returned an error: {0:?}.")]
    FdkDecoderError(fdk::AAC_DECODER_ERROR),

    #[error("The channel config in the aac audio is unsupported.")]
    UnsupportedChannelConfig,

    #[error("The aac decoder cannot decode chunks with kind {0:?}.")]
    UnsupportedChunkKind(EncodedChunkKind),
}

pub struct FdkAacDecoder;

impl FdkAacDecoder {
    pub(super) fn new(
        options: AacDecoderOptions,
        chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
        samples_sender: Sender<PipelineEvent<AudioSamplesBatch>>,
        input_id: InputId,
    ) -> Result<Self, AacDecoderError> {
        std::thread::Builder::new()
            .name(format!("fdk aac decoder {}", input_id.0))
            .spawn(move || {
                let _span = span!(
                    Level::INFO,
                    "fdk aac decoder",
                    input_id = input_id.to_string()
                )
                .entered();
                run_decoder_thread(options, samples_sender, chunks_receiver)
            })
            .unwrap();

        Ok(Self)
    }
}

fn run_decoder_thread(
    options: AacDecoderOptions,
    samples_sender: Sender<PipelineEvent<AudioSamplesBatch>>,
    chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
) {
    let mut decoder = None;
    let mut options = Some(options);

    for chunk in chunks_receiver {
        let chunk = match chunk {
            PipelineEvent::Data(chunk) => chunk,
            PipelineEvent::EOS => {
                break;
            }
        };

        let decoder = match &decoder {
            Some(d) => d,
            None => {
                decoder = match Decoder::new(
                    std::mem::take(&mut options)
                        .expect("AAC decoder initialization options should be present"),
                    &chunk,
                ) {
                    Ok(decoder) => Some(decoder),

                    Err(e) => {
                        // unfortunately, since this decoder needs to inspect the first data chunk
                        // to initialize, we cannot block in the main thread and wait for it to
                        // report a success or failure.
                        log::error!("Fatal AAC decoder error at initialization: {e}");
                        return;
                    }
                };

                decoder.as_ref().unwrap()
            }
        };

        let decoded_samples = match decoder.decode_chunk(chunk) {
            Ok(samples) => samples,
            Err(e) => {
                log::error!("Failed to decode AAC packet: {e}");
                continue;
            }
        };

        for batch in decoded_samples {
            if samples_sender.send(PipelineEvent::Data(batch)).is_err() {
                debug!("Failed to send audio samples from AAC decoder. Channel closed.");
                return;
            }
        }
    }
    if samples_sender.send(PipelineEvent::EOS).is_err() {
        debug!("Failed to send EOS from AAC decoder. Channel closed.")
    }
}

struct Decoder(*mut fdk::AAC_DECODER_INSTANCE);

impl Decoder {
    /// The encoded chunk used for initialization here still needs to be fed into `Decoder::decode_chunk` later
    fn new(
        options: AacDecoderOptions,
        first_chunk: &EncodedChunk,
    ) -> Result<Self, AacDecoderError> {
        let transport = if first_chunk.data[..4] == [b'A', b'D', b'I', b'F'] {
            fdk::TRANSPORT_TYPE_TT_MP4_ADIF
        } else if first_chunk.data[0] == 0xff && first_chunk.data[1] & 0xf0 == 0xf0 {
            fdk::TRANSPORT_TYPE_TT_MP4_ADTS
        } else {
            fdk::TRANSPORT_TYPE_TT_MP4_RAW
        };

        let dec = unsafe { fdk::aacDecoder_Open(transport, 1) };
        let dec = Decoder(dec);

        if let Some(config) = options.asc {
            let result = unsafe {
                fdk::aacDecoder_ConfigRaw(
                    dec.0,
                    &mut config.to_vec().as_mut_ptr(),
                    &(config.len() as u32),
                )
            };

            if result != fdk::AAC_DECODER_ERROR_AAC_DEC_OK {
                return Err(AacDecoderError::FdkDecoderError(result));
            }
        }

        let info = unsafe { *fdk::aacDecoder_GetStreamInfo(dec.0) };
        if info.channelConfig != 1 && info.channelConfig != 2 {
            return Err(AacDecoderError::UnsupportedChannelConfig);
        }

        Ok(dec)
    }

    fn decode_chunk(&self, chunk: EncodedChunk) -> Result<Vec<AudioSamplesBatch>, AacDecoderError> {
        if chunk.kind != EncodedChunkKind::Audio(crate::pipeline::AudioCodec::Aac) {
            return Err(AacDecoderError::UnsupportedChunkKind(chunk.kind));
        }

        let buffer_size = chunk.data.len() as u32;
        let mut bytes_valid = buffer_size;
        let mut buffer = chunk.data.to_vec();
        let mut output_buffer = Vec::new();

        while bytes_valid > 0 {
            // This fills the decoder with data.
            // It will adjust `bytes_valid` on its own based on how many bytes are left in the
            // buffer.
            let result = unsafe {
                fdk::aacDecoder_Fill(
                    self.0,
                    &mut buffer.as_mut_ptr(),
                    &buffer_size,
                    &mut bytes_valid,
                )
            };

            if result != fdk::AAC_DECODER_ERROR_AAC_DEC_OK {
                return Err(AacDecoderError::FdkDecoderError(result));
            }

            let info = unsafe { *fdk::aacDecoder_GetStreamInfo(self.0) };

            // The decoder should output `info.aacSamplesPerFrame` for each channel
            let mut decoded_samples: Vec<fdk::INT_PCM> =
                vec![0; (info.aacSamplesPerFrame * info.channelConfig) as usize];

            let result = unsafe {
                fdk::aacDecoder_DecodeFrame(
                    self.0,
                    decoded_samples.as_mut_ptr(),
                    decoded_samples.len() as i32,
                    0,
                )
            };

            if result == fdk::AAC_DECODER_ERROR_AAC_DEC_NOT_ENOUGH_BITS {
                // Need to put more data in
                continue;
            }

            if result != fdk::AAC_DECODER_ERROR_AAC_DEC_OK {
                return Err(AacDecoderError::FdkDecoderError(result));
            }

            let samples = match info.channelConfig {
                1 => AudioSamples::Mono(decoded_samples),
                2 => AudioSamples::Stereo(
                    decoded_samples
                        .chunks_exact(2)
                        .map(|c| (c[0], c[1]))
                        .collect(),
                ),

                _ => return Err(AacDecoderError::UnsupportedChannelConfig),
            };

            // We need the info before the decode call to get the channel info, but the sample rate
            // can change during `aacDecoder_DecodeFrame`
            let info = unsafe { *fdk::aacDecoder_GetStreamInfo(self.0) };

            output_buffer.push(AudioSamplesBatch {
                samples: samples.into(),
                start_pts: chunk.pts,
                sample_rate: info.sampleRate as u32,
            });
        }

        Ok(output_buffer)
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe {
            fdk::aacDecoder_Close(self.0);
        }
    }
}
