use std::{
    collections::VecDeque,
    mem::{self, MaybeUninit},
    os::raw::{c_int, c_void},
    ptr,
};

use bytes::BytesMut;
use crossbeam_channel::{bounded, Receiver, Sender};
use fdk_aac_sys as fdk;
use tracing::{span, Level};

use crate::{
    audio_mixer::{AudioChannels, AudioSamples, OutputSamples},
    error::OutputInitError,
    pipeline::{EncodedChunk, EncoderOutputEvent},
    queue::PipelineEvent,
};

#[derive(Debug, thiserror::Error)]
pub enum AacEncoderError {
    #[error("The internal fdk encoder returned an error: {0:?}.")]
    FdkEncoderError(fdk::AACENC_ERROR),
}

pub struct AacEncoder {
    samples_batch_sender: Sender<PipelineEvent<OutputSamples>>,
}

pub struct AacEncoderOptions {
    pub channels: AudioChannels,
}

impl AacEncoder {
    pub fn new(
        options: AacEncoderOptions,
        sample_rate: u32,
        packets_sender: Sender<EncoderOutputEvent>,
    ) -> Result<Self, OutputInitError> {
        let (samples_batch_sender, samples_batch_receiver) = bounded(2);
        let (init_result_sender, init_result_receiver) = bounded(1);

        std::thread::Builder::new()
            .name("Aac encoder thread".to_string())
            .spawn(move || {
                let _span = span!(Level::INFO, "Aac encoder thread",).entered();
                run_encoder_thread(
                    init_result_sender,
                    options,
                    sample_rate,
                    samples_batch_receiver,
                    packets_sender,
                );
            })
            .unwrap();

        init_result_receiver.recv().unwrap()?;

        Ok(Self {
            samples_batch_sender,
        })
    }

    pub fn samples_batch_sender(&self) -> &Sender<PipelineEvent<OutputSamples>> {
        &self.samples_batch_sender
    }
}

struct AacEncoderInner {
    encoder: *mut fdk::AACENCODER,
    input_buffer: VecDeque<i32>,
    output_buffer: Vec<u8>,
}

impl AacEncoderInner {
    fn new(options: AacEncoderOptions, sample_rate: u32) -> Result<Self, AacEncoderError> {
        let mut encoder = ptr::null_mut();
        let info;

        let channels = match options.channels {
            AudioChannels::Mono => 1,
            AudioChannels::Stereo => 2,
        };

        unsafe {
            check_fdk(fdk::aacEncOpen(&mut encoder as *mut _, 0, 2))?;
            check_fdk(fdk::aacEncoder_SetParam(
                encoder,
                fdk::AACENC_PARAM_AACENC_BITRATEMODE,
                5,
            ))?;
            check_fdk(fdk::aacEncoder_SetParam(
                encoder,
                fdk::AACENC_PARAM_AACENC_SAMPLERATE,
                sample_rate,
            ))?;
            check_fdk(fdk::aacEncoder_SetParam(
                encoder,
                fdk::AACENC_PARAM_AACENC_TRANSMUX,
                2,
            ))?;
            check_fdk(fdk::aacEncoder_SetParam(
                encoder,
                fdk::AACENC_PARAM_AACENC_SBR_MODE,
                0,
            ))?;
            check_fdk(fdk::aacEncoder_SetParam(
                encoder,
                fdk::AACENC_PARAM_AACENC_CHANNELMODE,
                channels,
            ))?;
            check_fdk(fdk::aacEncoder_SetParam(
                encoder,
                fdk::AACENC_PARAM_AACENC_AFTERBURNER,
                1,
            ))?;

            check_fdk(fdk::aacEncEncode(
                encoder,
                ptr::null(),
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
            ))?;

            let mut maybe_info = MaybeUninit::uninit();
            check_fdk(fdk::aacEncInfo(encoder, maybe_info.as_mut_ptr()))?;
            info = maybe_info.assume_init();
        }

        let output_buffer_size = info.maxOutBufBytes as usize;

        Ok(Self {
            encoder,
            input_buffer: VecDeque::new(),
            output_buffer: vec![0; output_buffer_size],
        })
    }

    fn enqueue_input(&mut self, samples: OutputSamples) {
        match samples.samples {
            AudioSamples::Mono(mono_samples) => {
                self.input_buffer
                    .extend(mono_samples.iter().map(|&s| s as i32));
            }
            AudioSamples::Stereo(stereo_samples) => {
                for (l, r) in stereo_samples {
                    self.input_buffer.push_back(l as i32);
                    self.input_buffer.push_back(r as i32);
                }
            }
        }
    }

    fn encode(&mut self) -> Result<EncodedChunk, AacEncoderError> {
        let output = BytesMut::new();

        while !self.input_buffer.is_empty() {
            // For safety, buffer descriptions are created right before calling `aacEncEncode``.
            // It's unsafe to use pointers obtained by calling `as_ptr()` and `as_ptr_mut()` after moving / reallocating the buffer.
            // If I would extract creating buffer and buffer description to a separate function, move could occur unpredictably
            // (depends on compiler), making pointers invalid and unsafe to use.
            let mut in_buf = input_buffer.as_ptr();
            let mut in_buf_ident: c_int = fdk::AACENC_BufferIdentifier_IN_AUDIO_DATA as c_int;
            let mut in_buf_size: c_int = input_buffer.len() as c_int;
            let mut in_buf_el_size: c_int = mem::size_of::<i16>() as c_int;

            let in_desc = fdk::AACENC_BufDesc {
                numBufs: 1,
                bufs: &mut in_buf as *mut _ as *mut *mut c_void,
                bufferIdentifiers: &mut in_buf_ident as *mut _,
                bufSizes: &mut in_buf_size as *mut _,
                bufElSizes: &mut in_buf_el_size as *mut _,
            };

            let mut out_buf = out_buffer.as_mut_ptr();
            let mut out_buf_ident: c_int = fdk::AACENC_BufferIdentifier_OUT_BITSTREAM_DATA as c_int;
            let mut out_buf_size: c_int = out_buffer.len() as c_int;
            let mut out_buf_el_size: c_int = mem::size_of::<i16>() as c_int;

            let out_desc = fdk::AACENC_BufDesc {
                numBufs: 1,
                bufs: &mut out_buf as *mut _ as *mut *mut c_void,
                bufferIdentifiers: &mut out_buf_ident as *mut _,
                bufSizes: &mut out_buf_size as *mut _,
                bufElSizes: &mut out_buf_el_size as *mut _,
            };

            unsafe {
                let mut out_args = mem::zeroed();

                check_fdk(fdk::aacEncEncode(
                    self.encoder,
                    &in_desc,
                    &out_desc,
                    &in_args,
                    &mut out_args,
                ))?;
            }
        }

        Ok(EncodedChunk {
            data: output.freeze(),
            pts: todo!(),
            dts: todo!(),
            kind: todo!(),
        })
    }
}

fn run_encoder_thread(
    init_result_sender: Sender<Result<(), OutputInitError>>,
    options: AacEncoderOptions,
    sample_rate: u32,
    samples_batch_receiver: Receiver<PipelineEvent<OutputSamples>>,
    packets_sender: Sender<EncoderOutputEvent>,
) {
    let encoder = match AacEncoderInner::new(options, sample_rate) {
        Ok(encoder) => {
            init_result_sender.send(Ok(())).unwrap();
            encoder
        }
        Err(err) => {
            init_result_sender.send(Err(err.into())).unwrap();
            return;
        }
    };

    for event in samples_batch_receiver {
        match event {
            PipelineEvent::Data(samples) => {
                // Encode samples
            }
            PipelineEvent::EOS => {
                packets_sender.send(EncoderOutputEvent::AudioEOS).unwrap();
                break;
            }
        }
    }
}

fn encode(
    encoder: *mut fdk::AACENCODER,
    samples: OutputSamples,
    out_buffer: &mut Vec<u8>,
) -> Result<(), AacEncoderError> {
    let in_args = fdk::AACENC_InArgs {
        numInSamples: samples.samples.len() as c_int,
        numAncBytes: 0,
    };

    let input_buffer = match samples.samples {
        AudioSamples::Mono(mono_samples) => mono_samples,
        AudioSamples::Stereo(stereo_samples) => stereo_samples
            .into_iter()
            .flat_map(|(l, r)| [l, r])
            .collect(),
    };

    // For safety, buffer descriptions are created right before calling `aacEncEncode``.
    // It's unsafe to use pointers obtained by calling `as_ptr()` and `as_ptr_mut()` after moving / reallocating the buffer.
    // If I would extract creating buffer and buffer description to a separate function, move could occur unpredictably
    // (depends on compiler), making pointers invalid and unsafe to use.
    let mut in_buf = input_buffer.as_ptr();
    let mut in_buf_ident: c_int = fdk::AACENC_BufferIdentifier_IN_AUDIO_DATA as c_int;
    let mut in_buf_size: c_int = input_buffer.len() as c_int;
    let mut in_buf_el_size: c_int = mem::size_of::<i16>() as c_int;

    let in_desc = fdk::AACENC_BufDesc {
        numBufs: 1,
        bufs: &mut in_buf as *mut _ as *mut *mut c_void,
        bufferIdentifiers: &mut in_buf_ident as *mut _,
        bufSizes: &mut in_buf_size as *mut _,
        bufElSizes: &mut in_buf_el_size as *mut _,
    };

    let mut out_buf = out_buffer.as_mut_ptr();
    let mut out_buf_ident: c_int = fdk::AACENC_BufferIdentifier_OUT_BITSTREAM_DATA as c_int;
    let mut out_buf_size: c_int = out_buffer.len() as c_int;
    let mut out_buf_el_size: c_int = mem::size_of::<i16>() as c_int;

    let out_desc = fdk::AACENC_BufDesc {
        numBufs: 1,
        bufs: &mut out_buf as *mut _ as *mut *mut c_void,
        bufferIdentifiers: &mut out_buf_ident as *mut _,
        bufSizes: &mut out_buf_size as *mut _,
        bufElSizes: &mut out_buf_el_size as *mut _,
    };

    unsafe {
        let mut out_args = mem::zeroed();

        check_fdk(fdk::aacEncEncode(
            encoder,
            &in_desc,
            &out_desc,
            &in_args,
            &mut out_args,
        ))?;
    }

    Ok(())
}

fn init_encoder(
    options: AacEncoderOptions,
    sample_rate: u32,
) -> Result<(*mut fdk::AACENCODER, fdk::AACENC_InfoStruct), AacEncoderError> {
    let mut encoder = ptr::null_mut();
    let info;

    let channels = match options.channels {
        AudioChannels::Mono => 1,
        AudioChannels::Stereo => 2,
    };

    unsafe {
        check_fdk(fdk::aacEncOpen(&mut encoder as *mut _, 0, 2))?;
        check_fdk(fdk::aacEncoder_SetParam(
            encoder,
            fdk::AACENC_PARAM_AACENC_BITRATEMODE,
            5,
        ))?;
        check_fdk(fdk::aacEncoder_SetParam(
            encoder,
            fdk::AACENC_PARAM_AACENC_SAMPLERATE,
            sample_rate,
        ))?;
        check_fdk(fdk::aacEncoder_SetParam(
            encoder,
            fdk::AACENC_PARAM_AACENC_TRANSMUX,
            2,
        ))?;
        check_fdk(fdk::aacEncoder_SetParam(
            encoder,
            fdk::AACENC_PARAM_AACENC_SBR_MODE,
            0,
        ))?;
        check_fdk(fdk::aacEncoder_SetParam(
            encoder,
            fdk::AACENC_PARAM_AACENC_CHANNELMODE,
            channels,
        ))?;
        check_fdk(fdk::aacEncoder_SetParam(
            encoder,
            fdk::AACENC_PARAM_AACENC_AFTERBURNER,
            1,
        ))?;

        check_fdk(fdk::aacEncEncode(
            encoder,
            ptr::null(),
            ptr::null(),
            ptr::null(),
            ptr::null_mut(),
        ))?;

        let mut maybe_info = MaybeUninit::uninit();
        check_fdk(fdk::aacEncInfo(encoder, maybe_info.as_mut_ptr()))?;
        info = maybe_info.assume_init();
    }

    Ok((encoder, info))
}

fn check_fdk(result: fdk::AACENC_ERROR) -> Result<(), AacEncoderError> {
    if result == fdk::AACENC_ERROR_AACENC_OK {
        Ok(())
    } else {
        Err(AacEncoderError::FdkEncoderError(result))
    }
}
