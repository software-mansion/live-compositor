use std::{path::PathBuf, ptr};

use compositor_render::OutputId;
use crossbeam_channel::Receiver;
use ffmpeg_next::{
    ffi::{AVChannelLayout, AVChannelOrder},
    Packet,
};
use log::error;

use crate::{
    audio_mixer::AudioChannels,
    error::OutputInitError,
    pipeline::{AudioCodec, EncodedChunk, EncodedChunkKind, EncoderOutputEvent, VideoCodec},
};

#[derive(Debug, Clone)]
pub struct Mp4OutputOptions {
    pub output_path: PathBuf,
    pub video: Option<Mp4VideoTrack>,
    pub audio: Option<Mp4AudioTrack>,
}

#[derive(Debug, Clone)]
pub struct Mp4VideoTrack {
    pub codec: VideoCodec,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Mp4AudioTrack {
    pub codec: AudioCodec,
    pub channels: AudioChannels,
}

pub enum Mp4OutputVideoTrack {
    H264 { width: u32, height: u32 },
}

pub struct Mp4WriterOptions {
    pub output_path: PathBuf,
    pub video: Option<Mp4OutputVideoTrack>,
}

pub struct Mp4FileWriter;

impl Mp4FileWriter {
    pub fn new(
        output_id: &OutputId,
        options: Mp4OutputOptions,
        packets_receiver: Receiver<EncoderOutputEvent>,
        sample_rate: u32,
    ) -> Result<Self, OutputInitError> {
        if options.output_path.exists() {
            return Err(OutputInitError::Mp4PathExist {
                path: options.output_path.to_string_lossy().into_owned(),
            });
        }

        let (mut output_ctx, video_stream, audio_stream) =
            Self::init_ffmpeg_output(options, sample_rate)?;
        let mut received_video_eos = video_stream.as_ref().map(|_| false);
        let mut received_audio_eos = audio_stream.as_ref().map(|_| false);

        std::thread::Builder::new()
            .name(format!("mp4 writer thread for output {}", output_id))
            .spawn(move || {
                for packet in packets_receiver {
                    match packet {
                        EncoderOutputEvent::Data(chunk) => {
                            Self::write_chunk(chunk, &video_stream, &audio_stream, &mut output_ctx);
                        }
                        EncoderOutputEvent::VideoEOS => match received_video_eos {
                            Some(false) => received_video_eos = Some(true),
                            Some(true) => {
                                error!("Received multiple video EOS events");
                            }
                            None => {
                                error!("Received video EOS event on non video output");
                            }
                        },
                        EncoderOutputEvent::AudioEOS => match received_audio_eos {
                            Some(false) => received_audio_eos = Some(true),
                            Some(true) => {
                                error!("Received multiple audio EOS events");
                            }
                            None => {
                                error!("Received audio EOS event on non audio output");
                            }
                        },
                    };

                    if received_video_eos.unwrap_or(true) && received_audio_eos.unwrap_or(true) {
                        if let Err(err) = output_ctx.write_trailer() {
                            error!("Failed to write trailer to mp4 file: {}", err);
                        };
                        break;
                    }
                }
            })
            .unwrap();

        Ok(Mp4FileWriter)
    }

    fn init_ffmpeg_output(
        options: Mp4OutputOptions,
        sample_rate: u32,
    ) -> Result<
        (
            ffmpeg_next::format::context::Output,
            Option<Stream>,
            Option<Stream>,
        ),
        OutputInitError,
    > {
        let mut output_ctx = ffmpeg_next::format::output_as(&options.output_path, "mp4")
            .map_err(OutputInitError::FfmpegMp4Error)?;

        let mut stream_count = 0;

        let video_stream = options
            .video
            .map(|v| {
                const VIDEO_TIME_BASE: i32 = 90000;

                let codec = match v.codec {
                    VideoCodec::H264 => ffmpeg_next::codec::Id::H264,
                };

                let mut stream = output_ctx
                    .add_stream(ffmpeg_next::codec::Id::H264)
                    .map_err(OutputInitError::FfmpegMp4Error)?;

                stream.set_time_base(ffmpeg_next::Rational::new(1, VIDEO_TIME_BASE));

                unsafe {
                    (*(*stream.as_mut_ptr()).codecpar).codec_id = codec.into();
                    (*(*stream.as_mut_ptr()).codecpar).codec_type =
                        ffmpeg_next::ffi::AVMediaType::AVMEDIA_TYPE_VIDEO;
                    (*(*stream.as_mut_ptr()).codecpar).width = v.width as i32;
                    (*(*stream.as_mut_ptr()).codecpar).height = v.height as i32;
                }

                let id = stream_count;
                stream_count += 1;

                Ok::<Stream, OutputInitError>(Stream {
                    id,
                    time_base: VIDEO_TIME_BASE as f64,
                })
            })
            .transpose()?;

        let audio_stream = options
            .audio
            .map(|a| {
                let codec = match a.codec {
                    AudioCodec::Aac => ffmpeg_next::codec::Id::AAC,
                    AudioCodec::Opus => ffmpeg_next::codec::Id::OPUS,
                };
                let channels = match a.channels {
                    AudioChannels::Mono => 1,
                    AudioChannels::Stereo => 2,
                };

                let mut stream = output_ctx
                    .add_stream(codec)
                    .map_err(OutputInitError::FfmpegMp4Error)?;

                // If audio time base doesn't match sample rate, ffmpeg muxer produces incorrect timestamps.
                stream.set_time_base(ffmpeg_next::Rational::new(1, sample_rate as i32));

                unsafe {
                    (*(*stream.as_mut_ptr()).codecpar).codec_id = codec.into();
                    (*(*stream.as_mut_ptr()).codecpar).codec_type =
                        ffmpeg_next::ffi::AVMediaType::AVMEDIA_TYPE_AUDIO;
                    (*(*stream.as_mut_ptr()).codecpar).sample_rate = sample_rate as i32;
                    (*(*stream.as_mut_ptr()).codecpar).ch_layout = AVChannelLayout {
                        nb_channels: channels,
                        order: AVChannelOrder::AV_CHANNEL_ORDER_UNSPEC,
                        // This value is ignored when order is AV_CHANNEL_ORDER_UNSPEC
                        u: ffmpeg_next::ffi::AVChannelLayout__bindgen_ty_1 { mask: 0 },
                        // Field doc: "For some private data of the user."
                        opaque: ptr::null_mut(),
                    };
                }

                let id = stream_count;
                stream_count += 1;

                Ok::<Stream, OutputInitError>(Stream {
                    id,
                    time_base: sample_rate as f64,
                })
            })
            .transpose()?;

        output_ctx
            .write_header()
            .map_err(OutputInitError::FfmpegMp4Error)?;

        Ok((output_ctx, video_stream, audio_stream))
    }

    fn write_chunk(
        chunk: EncodedChunk,
        video_stream: &Option<Stream>,
        audio_stream: &Option<Stream>,
        output_ctx: &mut ffmpeg_next::format::context::Output,
    ) {
        let packet = Self::create_packet(chunk, video_stream, audio_stream);
        if let Some(packet) = packet {
            if let Err(err) = packet.write(output_ctx) {
                error!("Failed to write packet to mp4 file: {}", err);
            }
        }
    }

    fn create_packet(
        chunk: EncodedChunk,
        video_stream: &Option<Stream>,
        audio_stream: &Option<Stream>,
    ) -> Option<Packet> {
        let (stream_id, timebase) = match chunk.kind {
            EncodedChunkKind::Video(_) => match video_stream {
                Some(Stream { id, time_base }) => Some((*id, *time_base)),
                None => {
                    error!("Failed to create packet for video chunk. No video stream registered on init.");
                    None
                }
            },
            EncodedChunkKind::Audio(_) => match audio_stream {
                Some(Stream { id, time_base }) => Some((*id, *time_base)),
                None => {
                    error!("Failed to create packet for audio chunk. No audio stream registered on init.");
                    None
                }
            },
        }?;

        let mut packet = ffmpeg_next::Packet::copy(&chunk.data);
        packet.set_pts(Some((chunk.pts.as_secs_f64() * timebase) as i64));
        let dts = chunk.dts.unwrap_or(chunk.pts);
        packet.set_dts(Some((dts.as_secs_f64() * timebase) as i64));
        packet.set_time_base(ffmpeg_next::Rational::new(1, timebase as i32));
        packet.set_stream(stream_id);

        Some(packet)
    }
}

#[derive(Debug, Clone)]
struct Stream {
    id: usize,
    time_base: f64,
}
