use std::ops::ControlFlow;
use std::sync::Arc;

use compositor_render::InputId;
use crossbeam_channel::{Receiver, Sender};
use fdk_aac_sys as fdk;
use log::warn;
use tracing::{debug, error, span, Level};

use crate::{
    pipeline::structs::{EncodedChunk, EncodedChunkKind},
    queue::PipelineEvent,
};

use super::{AacDecoderOptions, DecodedAudioInputInfo, DecodedSamples};

#[derive(Debug, thiserror::Error)]
pub enum AacDecoderError {
    #[error("The internal fdk decoder returned an error: {0:?}.")]
    FdkDecoderError(fdk::AAC_DECODER_ERROR),

    #[error("The channel config in the aac audio is unsupported.")]
    UnsupportedChannelConfig,

    #[error("The aac decoder cannot decode chunks with kind {0:?}.")]
    UnsupportedChunkKind(EncodedChunkKind),

    #[error("The aac decoder cannot decode chunks with sample rate {0}.")]
    UnsupportedSampleRate(i32),

    #[error("The aac decoder thread start failed.")]
    DecoderStartFailure,
}

pub struct FdkAacDecoder;

impl FdkAacDecoder {
    pub(super) fn spawn(
        options: AacDecoderOptions,
        chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
        samples_sender: Sender<PipelineEvent<DecodedSamples>>,
        input_id: InputId,
    ) -> Result<DecodedAudioInputInfo, AacDecoderError> {
        let (result_sender, result_receiver) = crossbeam_channel::bounded(1);
        std::thread::Builder::new()
            .name(format!("fdk aac decoder {}", input_id.0))
            .spawn(move || {
                let _span = span!(
                    Level::INFO,
                    "fdk aac decoder",
                    input_id = input_id.to_string()
                )
                .entered();
                run_decoder_thread(options, chunks_receiver, samples_sender, result_sender)
            })
            .unwrap();

        let Ok(decoded_sample_rate) = result_receiver.recv() else {
            return Err(AacDecoderError::DecoderStartFailure);
        };
        let info = DecodedAudioInputInfo {
            decoded_sample_rate,
        };

        Ok(info)
    }
}

fn run_decoder_thread(
    options: AacDecoderOptions,
    chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
    samples_sender: Sender<PipelineEvent<DecodedSamples>>,
    result_sender: Sender<u32>,
) {
    let chunk = match chunks_receiver.recv() {
        Ok(PipelineEvent::Data(chunk)) => chunk,
        Ok(PipelineEvent::EOS) | Err(_) => {
            log::warn!("AAC decoder received no data and its input stream has ended");
            return;
        }
    };

    let decoder = match Decoder::new(options.clone(), &chunk) {
        Ok(decoder) => decoder,
        Err(e) => {
            // unfortunately, since this decoder needs to inspect the first data chunk
            // to initialize, we cannot block in the main thread and wait for it to
            // report a success or failure.
            log::error!("Fatal AAC decoder error at initialization: {e}");
            return;
        }
    };

    if let Err(err) = result_sender.send(decoder.sample_rate) {
        warn!("Failed to send decoder init result: {}", err);
    };

    if process_chunk(chunk, &decoder, &samples_sender).is_break() {
        return;
    }

    for chunk in chunks_receiver {
        let chunk = match chunk {
            PipelineEvent::Data(chunk) => chunk,
            PipelineEvent::EOS => {
                break;
            }
        };

        if process_chunk(chunk, &decoder, &samples_sender).is_break() {
            break;
        }
    }
}

fn process_chunk(
    chunk: EncodedChunk,
    decoder: &Decoder,
    sender: &Sender<PipelineEvent<DecodedSamples>>,
) -> ControlFlow<()> {
    let decoded_samples = match decoder.decode_chunk(chunk) {
        Ok(samples) => samples,
        Err(e) => {
            log::error!("Failed to decode AAC packet: {e}");
            return ControlFlow::Continue(());
        }
    };

    for batch in decoded_samples {
        if sender.send(PipelineEvent::Data(batch)).is_err() {
            debug!("Failed to send audio samples from AAC decoder. Channel closed.");

            if sender.send(PipelineEvent::EOS).is_err() {
                debug!("Failed to send EOS from AAC decoder. Channel closed.")
            }

            return ControlFlow::Break(());
        }
    }

    ControlFlow::Continue(())
}

struct Decoder {
    instance: *mut fdk::AAC_DECODER_INSTANCE,
    sample_rate: u32,
}

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

        let instance = unsafe { fdk::aacDecoder_Open(transport, 1) };

        if let Some(config) = options.asc {
            let result = unsafe {
                fdk::aacDecoder_ConfigRaw(
                    instance,
                    &mut config.to_vec().as_mut_ptr(),
                    &(config.len() as u32),
                )
            };

            if result != fdk::AAC_DECODER_ERROR_AAC_DEC_OK {
                return Err(AacDecoderError::FdkDecoderError(result));
            }
        }

        let info = unsafe { *fdk::aacDecoder_GetStreamInfo(instance) };
        let aac_sample_rate = info.aacSampleRate;
        let sample_rate = if aac_sample_rate > 0 {
            aac_sample_rate as u32
        } else {
            return Err(AacDecoderError::UnsupportedSampleRate(aac_sample_rate));
        };
        if info.channelConfig != 1 && info.channelConfig != 2 {
            return Err(AacDecoderError::UnsupportedChannelConfig);
        }

        Ok(Decoder {
            instance,
            sample_rate,
        })
    }

    fn decode_chunk(&self, chunk: EncodedChunk) -> Result<Vec<DecodedSamples>, AacDecoderError> {
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
                    self.instance,
                    &mut buffer.as_mut_ptr(),
                    &buffer_size,
                    &mut bytes_valid,
                )
            };

            if result != fdk::AAC_DECODER_ERROR_AAC_DEC_OK {
                return Err(AacDecoderError::FdkDecoderError(result));
            }

            let info = unsafe { *fdk::aacDecoder_GetStreamInfo(self.instance) };

            // The decoder should output `info.aacSamplesPerFrame` for each channel
            let mut decoded_samples: Vec<fdk::INT_PCM> =
                vec![0; (info.aacSamplesPerFrame * info.channelConfig) as usize];

            let result = unsafe {
                fdk::aacDecoder_DecodeFrame(
                    self.instance,
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
                1 => Arc::new(decoded_samples.iter().map(|s| (*s, *s)).collect()),
                2 => Arc::new(
                    decoded_samples
                        .chunks_exact(2)
                        .map(|c| (c[0], c[1]))
                        .collect(),
                ),
                _ => return Err(AacDecoderError::UnsupportedChannelConfig),
            };

            // Sample rate can change after decoding
            let info = unsafe { *fdk::aacDecoder_GetStreamInfo(self.instance) };
            let sample_rate = if info.sampleRate > 0 {
                info.sampleRate as u32
            } else {
                error!(
                    "Unexpected sample rate of decoded AAC audio: {}",
                    info.sampleRate
                );
                0
            };

            output_buffer.push(DecodedSamples {
                samples,
                start_pts: chunk.pts,
                sample_rate,
            })
        }

        Ok(output_buffer)
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe {
            fdk::aacDecoder_Close(self.instance);
        }
    }
}
