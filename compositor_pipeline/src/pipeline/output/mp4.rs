use std::path::PathBuf;

use compositor_render::OutputId;
use crossbeam_channel::Receiver;
use ffmpeg_next::Packet;
use log::{error, info};

use crate::{
    error::{OutputInitError, RegisterOutputError},
    pipeline::{
        encoder::{AudioEncoderOptions, VideoEncoderOptions},
        AudioCodec, EncodedChunk, EncoderOutputEvent, VideoCodec,
    },
};

#[derive(Debug, Clone)]
pub struct Mp4OutputOptions {
    pub output_path: PathBuf,
    pub video: Option<Mp4VideoTrack>,
    pub audio: Option<AudioCodec>,
}

#[derive(Debug, Clone)]
pub struct Mp4VideoTrack {
    pub codec: VideoCodec,
    pub width: u32,
    pub height: u32,
}

pub enum Mp4OutputVideoTrack {
    H264 { width: u32, height: u32 },
}

pub struct Mp4WriterOptions {
    pub output_path: PathBuf,
    pub video: Option<Mp4OutputVideoTrack>,
}

impl Mp4WriterOptions {
    pub fn new(
        output_id: &OutputId,
        output_path: PathBuf,
        video: &Option<VideoEncoderOptions>,
        audio: &Option<AudioEncoderOptions>,
    ) -> Result<Self, RegisterOutputError> {
        match (video, audio) {
            (Some(_), Some(_)) | (None, Some(_)) => {
                Err(RegisterOutputError::Mp4AudioNotSupported(output_id.clone()))
            }
            (Some(video), None) => {
                let mp4_video_opt = match video {
                    VideoEncoderOptions::H264(opt) => Mp4OutputVideoTrack::H264 {
                        width: opt.resolution.width as u32,
                        height: opt.resolution.height as u32,
                    },
                };

                Ok(Mp4WriterOptions {
                    video: Some(mp4_video_opt),
                    output_path: output_path.clone(),
                })
            }
            (None, None) => Err(RegisterOutputError::NoVideoAndAudio(output_id.clone())),
        }
    }
}

pub struct Mp4FileWriter;

impl Mp4FileWriter {
    pub fn new(
        output_id: &OutputId,
        options: Mp4OutputOptions,
        packets_receiver: Receiver<EncoderOutputEvent>,
    ) -> Result<Self, OutputInitError> {
        let mut output_ctx = ffmpeg_next::format::output_as(&options.output_path, "mp4")
            .map_err(OutputInitError::FfmpegMp4Error)?;

        if let Some(video) = options.video {
            let mut stream = output_ctx
                .add_stream(ffmpeg_next::codec::Id::H264)
                .map_err(OutputInitError::FfmpegMp4Error)?;

            // I won't even pretend to understand what's going on here
            unsafe {
                (*(*stream.as_mut_ptr()).codecpar).codec_id = ffmpeg_next::codec::Id::H264.into();
                (*(*stream.as_mut_ptr()).codecpar).codec_type =
                    ffmpeg_next::ffi::AVMediaType::AVMEDIA_TYPE_VIDEO;
                (*(*stream.as_mut_ptr()).codecpar).width = video.width as i32;
                (*(*stream.as_mut_ptr()).codecpar).height = video.height as i32;
            }
        }

        output_ctx
            .write_header()
            .map_err(OutputInitError::FfmpegMp4Error)?;
        // TODO handle audio

        std::thread::Builder::new()
            .name(format!("mp4 writer thread for output {}", output_id))
            .spawn(move || {
                for packet in packets_receiver {
                    match packet {
                        EncoderOutputEvent::Data(chunk) => {
                            if let Err(err) =
                                Self::packet_from_encoded_chunk(chunk).write(&mut output_ctx)
                            {
                                error!("Failed to write packet to mp4 file: {}", err);
                            }
                        }
                        EncoderOutputEvent::AudioEOS => break,
                        EncoderOutputEvent::VideoEOS => {
                            match output_ctx.write_trailer() {
                                Ok(()) => {
                                    info!("Successfully wrote trailer to mp4 file");
                                }
                                Err(err) => {
                                    error!("Failed to write trailer to mp4 file: {}", err);
                                }
                            };
                            break;
                        }
                    };
                }
            })
            .unwrap();

        Ok(Mp4FileWriter)
    }

    fn packet_from_encoded_chunk(chunk: EncodedChunk) -> Packet {
        let mut packet = ffmpeg_next::Packet::copy(&chunk.data);
        packet.set_pts(Some((chunk.pts.as_secs_f64() * 90_000.0).round() as i64));
        packet.set_dts(
            chunk
                .dts
                .map(|dts| (dts.as_secs_f64() * 90_000.0) as i64),
        );

        packet
    }
}
