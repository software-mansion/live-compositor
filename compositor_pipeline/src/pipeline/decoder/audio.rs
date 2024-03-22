use compositor_render::InputId;
use crossbeam_channel::{bounded, Receiver, Sender};
use log::error;

use crate::{audio_mixer::InputSamples, error::DecoderInitError, pipeline::structs::EncodedChunk, queue::PipelineEvent};

use self::{fdk_aac_decoder::{AacDecoder, AacDecoderError}, opus_decoder::OpusDecoder};

use super::{resampler::{FftResampler, PassthroughResampler, ResamplerT}, AudioDecoderOptions, DecodedAudioFormat, DecodedSamples};

pub mod fdk_aac_decoder;
pub mod opus_decoder;

#[derive(Debug, thiserror::Error)]
pub enum DecodingError {
    #[error(transparent)]
    OpusError(#[from] opus::Error),
    #[error(transparent)]
    AacDecoder(#[from] AacDecoderError),
}

pub(super) trait AudioDecoderT {
    fn decode(&mut self, encoded_chunk: EncodedChunk)
        -> Result<Vec<DecodedSamples>, DecodingError>;

    fn decoded_format(&self) -> DecodedAudioFormat;
}

pub fn spawn_audio_decoder(
    opts: AudioDecoderOptions,
    output_sample_rate: u32,
    chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
    samples_sender: Sender<PipelineEvent<InputSamples>>,
    input_id: InputId,
) -> Result<(), DecoderInitError> {
    let (decoder_init_result_sender, decoder_init_result_receiver) = bounded(0);
    std::thread::Builder::new()
        .name(format!("Decoder thread for input {}", input_id.clone()))
        .spawn(move || {
            run_decoder_thread(
                opts,
                output_sample_rate,
                chunks_receiver,
                samples_sender,
                decoder_init_result_sender,
            );
        })
        .unwrap();

    match decoder_init_result_receiver.recv() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(init_err)) => Err(init_err),
        Err(_recv_err) => Err(DecoderInitError::CannotReadInitResult),
    }
}

/// init_result_sender sends:
/// - true init result for Opus
/// - always ok for AAC (aac sample rate is unknown at register time, first chunk is need to determine it)
fn run_decoder_thread(
    opts: AudioDecoderOptions,
    output_sample_rate: u32,
    chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
    samples_sender: Sender<PipelineEvent<InputSamples>>,
    init_result_sender: Sender<Result<(), DecoderInitError>>,
) {
    // AAC decoder output can have any sample rate, therefore resampling it to output sample rate is needed.
    // In Opus decoder we can configure output sampler rate and decoder performs resampling, however if output sample
    // rate is not supported by Opus (e.g. 44100Hz), then resampling is also needed.

    // In case of AAC decoder, decoded samples rate is unknown on registration
    // (decoder has to parse the first received chunk to determine it).
    // This means true AAC init result can't be send back.
    // Registering input can't be blocked on init_result.recv() by this thread.
    // It means that AAC decoder output sample rate is unknown at register
    // and AAC decoder init error and resampler init error won't be send back.

    let send_result = |result: Result<(), DecoderInitError>| {
        if init_result_sender.send(result).is_err() {
            error!("Failed to send decoder init result.");
        }
    };

    match opts {
        AudioDecoderOptions::Opus(opus_decoder_opts) => {
            // Opus decoder initialization doesn't require input stream data,
            // so this can wait and send init result
            let init_res = OpusDecoder::new(opus_decoder_opts, output_sample_rate)
                .map(|mut decoder| {
                    create_resampler_run_decoding::<OpusDecoder>(
                        chunks_receiver,
                        &mut decoder,
                        samples_sender,
                        output_sample_rate,
                    )
                })
                .and_then(|res| res);
            send_result(init_res)
        }
        AudioDecoderOptions::Aac(aac_decoder_opts) => {
            // unfortunately, since this decoder needs to inspect the first data chunk
            // to initialize, we cannot block in the main thread and wait for it to
            // report a success or failure.
            send_result(Ok(()));
            let first_chunk = match chunks_receiver.recv() {
                Ok(PipelineEvent::Data(first_chunk)) => first_chunk,
                Ok(PipelineEvent::EOS) => {
                    return;
                }
                Err(_) => {
                    error!("Failed to read the first chunk from input to initialize decoder.");
                    return;
                }
            };
            let init_res = AacDecoder::new(aac_decoder_opts, &first_chunk)
                .map(|mut decoder| {
                    create_resampler_run_decoding::<AacDecoder>(
                        chunks_receiver,
                        &mut decoder,
                        samples_sender,
                        output_sample_rate,
                    )
                })
                .and_then(|res| res);

            if let Err(err) = init_res {
                error!("Fatal AAC decoder initialization error. {}", err)
            }
        }
    }
}

fn create_resampler_run_decoding<Decoder: AudioDecoderT>(
    chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
    decoder: &mut dyn AudioDecoderT,
    samples_sender: Sender<PipelineEvent<InputSamples>>,
    output_sample_rate: u32,
) -> Result<(), DecoderInitError> {
    let decoded_format = decoder.decoded_format();
    if decoded_format.sample_rate == output_sample_rate {
        let mut resampler = PassthroughResampler::new(decoded_format, output_sample_rate)?;
        run_decoding::<Decoder, PassthroughResampler>(
            chunks_receiver,
            decoder,
            &mut resampler,
            samples_sender,
        );
    } else {
        let mut resampler = FftResampler::new(decoded_format, output_sample_rate)?;
        run_decoding::<Decoder, FftResampler>(
            chunks_receiver,
            decoder,
            &mut resampler,
            samples_sender,
        );
    }

    Ok(())
}

fn run_decoding<Decoder: AudioDecoderT, Resampler: ResamplerT>(
    chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
    decoder: &mut dyn AudioDecoderT,
    resampler: &mut dyn ResamplerT,
    samples_sender: Sender<PipelineEvent<InputSamples>>,
) {
    for event in chunks_receiver {
        let PipelineEvent::Data(encoded_chunk) = event else {
                return;
            };

        let decoded_samples_vec = match decoder.decode(encoded_chunk) {
            Ok(decoded_samples) => decoded_samples,
            Err(err) => {
                error!("Failed to decode samples. Error: {}", err);
                continue;
            }
        };

        for decoded_samples in decoded_samples_vec {
            for input_samples in resampler.resample(decoded_samples) {
                if samples_sender
                    .send(PipelineEvent::Data(input_samples))
                    .is_err()
                {
                    error!("Failed to send decoded input samples.")
                }
            }
        }
    }
}
