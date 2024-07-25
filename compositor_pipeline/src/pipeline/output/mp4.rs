use std::path::PathBuf;

use compositor_render::OutputId;
use crossbeam_channel::Receiver;
use ffmpeg_next::Packet;
use log::error;

use crate::{
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
    ) -> Result<Self, OutputInitError> {
        if options.output_path.exists() {
            return Err(OutputInitError::Mp4PathExist {
                path: options.output_path.to_string_lossy().into_owned(),
            });
        }

        let (mut output_ctx, video_stream_id, audio_stream_id) = Self::init_ffmpeg_output(options)?;
        let mut playing_streams = [video_stream_id, audio_stream_id]
            .into_iter()
            .filter(Option::is_some)
            .count();

        std::thread::Builder::new()
            .name(format!("mp4 writer thread for output {}", output_id))
            .spawn(move || {
                for packet in packets_receiver {
                    match packet {
                        EncoderOutputEvent::Data(chunk) => {
                            Self::write_chunk(
                                chunk,
                                video_stream_id,
                                audio_stream_id,
                                &mut output_ctx,
                            );
                        }
                        EncoderOutputEvent::AudioEOS => playing_streams -= 1,
                        EncoderOutputEvent::VideoEOS => playing_streams -= 1,
                    };

                    if playing_streams == 0 {
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
    ) -> Result<
        (
            ffmpeg_next::format::context::Output,
            Option<usize>,
            Option<usize>,
        ),
        OutputInitError,
    > {
        let mut output_ctx = ffmpeg_next::format::output_as(&options.output_path, "mp4")
            .map_err(OutputInitError::FfmpegMp4Error)?;

        let mut stream_count = 0;

        let video_stream_id = options
            .video
            .map(|v| {
                let codec = match v.codec {
                    VideoCodec::H264 => ffmpeg_next::codec::Id::H264,
                };

                let mut stream = output_ctx
                    .add_stream(ffmpeg_next::codec::Id::H264)
                    .map_err(OutputInitError::FfmpegMp4Error)?;

                stream.set_time_base(ffmpeg_next::Rational::new(1, 1_000_000));

                unsafe {
                    (*(*stream.as_mut_ptr()).codecpar).codec_id = codec.into();
                    (*(*stream.as_mut_ptr()).codecpar).codec_type =
                        ffmpeg_next::ffi::AVMediaType::AVMEDIA_TYPE_VIDEO;
                    (*(*stream.as_mut_ptr()).codecpar).width = v.width as i32;
                    (*(*stream.as_mut_ptr()).codecpar).height = v.height as i32;
                }

                let id = stream_count;
                stream_count += 1;

                Ok::<usize, OutputInitError>(id)
            })
            .transpose()?;

        let audio_stream_id = options
            .audio
            .map(|a| {
                let codec = match a.codec {
                    AudioCodec::Aac => ffmpeg_next::codec::Id::AAC,
                    AudioCodec::Opus => ffmpeg_next::codec::Id::OPUS,
                };

                let mut stream = output_ctx
                    .add_stream(codec)
                    .map_err(OutputInitError::FfmpegMp4Error)?;

                stream.set_time_base(ffmpeg_next::Rational::new(1, 1_000_000));

                unsafe {
                    (*(*stream.as_mut_ptr()).codecpar).codec_id = codec.into();
                    (*(*stream.as_mut_ptr()).codecpar).codec_type =
                        ffmpeg_next::ffi::AVMediaType::AVMEDIA_TYPE_AUDIO;
                    (*(*stream.as_mut_ptr()).codecpar).sample_rate = 48_000;
                }

                let id = stream_count;
                stream_count += 1;

                Ok::<usize, OutputInitError>(id)
            })
            .transpose()?;

        output_ctx
            .write_header()
            .map_err(OutputInitError::FfmpegMp4Error)?;

        Ok((output_ctx, video_stream_id, audio_stream_id))
    }

    fn write_chunk(
        chunk: EncodedChunk,
        video_stream_id: Option<usize>,
        audio_stream_id: Option<usize>,
        output_ctx: &mut ffmpeg_next::format::context::Output,
    ) {
        let packet = Self::create_packet(chunk, video_stream_id, audio_stream_id);
        if let Some(packet) = packet {
            if let Err(err) = packet.write(output_ctx) {
                error!("Failed to write packet to mp4 file: {}", err);
            }
        }
    }

    fn create_packet(
        chunk: EncodedChunk,
        video_stream_id: Option<usize>,
        audio_stream_id: Option<usize>,
    ) -> Option<Packet> {
        let stream_id = match chunk.kind {
            EncodedChunkKind::Video(_) => match video_stream_id {
                Some(id) => Some(id),
                None => {
                    error!("Failed to create packet for video chunk. No video stream registered on init.");
                    None
                }
            },
            EncodedChunkKind::Audio(_) => match audio_stream_id {
                Some(id) => Some(id),
                None => {
                    error!("Failed to create packet for audio chunk. No audio stream registered on init.");
                    None
                }
            },
        }?;

        let mut packet = ffmpeg_next::Packet::copy(&chunk.data);
        // Assume time 1 / 1_000_000
        packet.set_pts(Some(chunk.pts.as_micros() as i64));
        packet.set_dts(chunk.dts.map(|dts| dts.as_micros() as i64));
        packet.set_stream(stream_id);

        Some(packet)
    }
}
