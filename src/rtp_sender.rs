use anyhow::{anyhow, Result};
use crossbeam_channel::{Receiver, Sender};
use log::{error, warn};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, thread};

use compositor_common::{scene::Resolution, Frame};
use compositor_pipeline::pipeline::PipelineOutput;
use ffmpeg_next::{
    codec::{self, Context, Id},
    format::{self, Pixel},
    frame, Dictionary, Rational,
};

pub struct RtpSender {
    sender: Sender<Frame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum EncoderPreset {
    Ultrafast,
    Superfast,
    Veryfast,
    Faster,
    Fast,
    #[default]
    Medium,
    Slow,
    Slower,
    Veryslow,
    Placebo,
}

impl EncoderPreset {
    fn to_str(&self) -> &'static str {
        match self {
            EncoderPreset::Ultrafast => "ultrafast",
            EncoderPreset::Superfast => "superfast",
            EncoderPreset::Veryfast => "veryfast",
            EncoderPreset::Faster => "faster",
            EncoderPreset::Fast => "fast",
            EncoderPreset::Medium => "medium",
            EncoderPreset::Slow => "slow",
            EncoderPreset::Slower => "slower",
            EncoderPreset::Veryslow => "veryslow",
            EncoderPreset::Placebo => "placebo",
        }
    }

    fn default_partitions(&self) -> &'static str {
        match self {
            EncoderPreset::Ultrafast => "none",
            EncoderPreset::Superfast => "i8x8,i4x4",
            EncoderPreset::Veryfast => "p8x8,b8x8,i8x8,i4x4",
            EncoderPreset::Faster => "p8x8,b8x8,i8x8,i4x4",
            EncoderPreset::Fast => "p8x8,b8x8,i8x8,i4x4",
            EncoderPreset::Medium => "p8x8,b8x8,i8x8,i4x4",
            EncoderPreset::Slow => "all",
            EncoderPreset::Slower => "all",
            EncoderPreset::Veryslow => "all",
            EncoderPreset::Placebo => "all",
        }
    }

    fn default_subq_mode(&self) -> &'static str {
        match self {
            EncoderPreset::Ultrafast => "0",
            EncoderPreset::Superfast => "1",
            EncoderPreset::Veryfast => "2",
            EncoderPreset::Faster => "4",
            EncoderPreset::Fast => "6",
            EncoderPreset::Medium => "7",
            EncoderPreset::Slow => "8",
            EncoderPreset::Slower => "9",
            EncoderPreset::Veryslow => "10",
            EncoderPreset::Placebo => "11",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EncoderSettings {
    #[serde(default)]
    preset: EncoderPreset,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Options {
    pub port: u16,
    pub ip: String,
    pub resolution: Resolution,
    pub encoder_settings: EncoderSettings,
}

impl RtpSender {
    pub fn new(options: Options) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        thread::spawn(move || RtpSender::run(options, receiver).unwrap());
        Self { sender }
    }

    fn run(opts: Options, receiver: Receiver<Frame>) -> Result<()> {
        let mut output_ctx = format::output_as(
            &PathBuf::from(format!(
                "rtp://{}:{}?rtcpport={}",
                opts.ip, opts.port, opts.port
            )),
            "rtp",
        )?;
        let h264_codec = codec::encoder::find(Id::H264).unwrap();
        let mut stream = output_ctx.add_stream(h264_codec)?;
        unsafe {
            (*(*stream.as_mut_ptr()).codecpar).codec_id = codec::Id::H264.into();
        }

        let mut encoder = Context::new().encoder().video()?;
        let pts_unit_secs = Rational::new(1, 90000);
        encoder.set_time_base(pts_unit_secs);
        encoder.set_format(Pixel::YUV420P);
        encoder.set_width(opts.resolution.width.try_into().unwrap());
        encoder.set_height(opts.resolution.height.try_into().unwrap());

        let mut encoder = encoder.open_as_with(
            h264_codec,
            // TODO: audit settings bellow
            // Those values are copied from somewhere, they have to be set because libx264
            // is throwing an error if it detects default ffmpeg settings.
            Dictionary::from_iter([
                ("preset", opts.encoder_settings.preset.to_str()),
                // Quality-based VBR (0-51)
                ("crf", "23"),
                // Override ffmpeg defaults from https://github.com/mirror/x264/blob/eaa68fad9e5d201d42fde51665f2d137ae96baf0/encoder/encoder.c#L674
                // QP curve compression - libx264 defaults to 0.6 (in case of tune=grain to 0.8)
                ("qcomp", "0.6"),
                //  Maximum motion vector search range - libx264 defaults to 16 (in case of placebo
                //  or veryslow preset to 24)
                ("me_range", "16"),
                // Max QP step - libx264 defaults to 4
                ("qdiff", "4"),
                // Min QP - libx264 defaults to 0
                ("qmin", "0"),
                // Max QP - libx264 defaults to QP_MAX = 69
                ("qmax", "69"),
                //  Maximum GOP (Group of Pictures) size - libx264 defaults to 250
                ("g", "250"),
                // QP factor between I and P frames - libx264 defaults to 1.4 (in case of tune=grain to 1.1)
                ("i_qfactor", "1.4"),
                // QP factor between P and B frames - libx264 defaults to 1.4 (in case of tune=grain to 1.1)
                ("f_pb_factor", "1.3"),
                // A comma-separated list of partitions to consider. Possible values: p8x8, p4x4, b8x8, i8x8, i4x4, none, all
                (
                    "partitions",
                    opts.encoder_settings.preset.default_partitions(),
                ),
                // Subpixel motion estimation and mode decision (decision quality: 1=fast, 11=best)
                ("subq", opts.encoder_settings.preset.default_subq_mode()),
            ]),
        )?;
        output_ctx.write_header()?;
        let mut packet = codec::packet::Packet::empty();
        let mut av_frame = frame::Video::new(
            Pixel::YUV420P,
            opts.resolution.width.try_into().unwrap(),
            opts.resolution.height.try_into().unwrap(),
        );
        for frame in receiver.iter() {
            if receiver.len() > 100 {
                warn!("Dropping frame: encoder queue is too long.",);
                continue;
            }
            if let Err(err) = frame_into_av(frame, &mut av_frame) {
                error!("Failed to construct AVFrame: {err}");
                continue;
            }
            encoder.send_frame(&av_frame).unwrap();
            while encoder.receive_packet(&mut packet).is_ok() {
                if let Err(err) = packet.write_interleaved(&mut output_ctx) {
                    error!("Failed to send rtp packets: {err}")
                }
            }
        }

        Ok(())
    }
}

impl PipelineOutput for RtpSender {
    fn send_frame(&self, frame: Frame) {
        self.sender.send(frame).unwrap();
    }
}

fn frame_into_av(frame: Frame, av_frame: &mut frame::Video) -> Result<()> {
    let expected_y_plane_size = av_frame.data(0).len();
    let expected_u_plane_size = av_frame.data(1).len();
    let expected_v_plane_size = av_frame.data(2).len();
    if expected_y_plane_size != frame.data.y_plane.len() {
        return Err(anyhow!(
            "Y plane is a wrong size, expected: {} received: {}",
            expected_y_plane_size,
            frame.data.y_plane.len()
        ));
    }
    if expected_u_plane_size != frame.data.u_plane.len() {
        return Err(anyhow!(
            "U plane is a wrong size, expected: {} received: {}",
            expected_u_plane_size,
            frame.data.u_plane.len()
        ));
    }
    if expected_v_plane_size != frame.data.v_plane.len() {
        return Err(anyhow!(
            "V plane is a wrong size, expected: {} received: {}",
            expected_v_plane_size,
            frame.data.v_plane.len()
        ));
    }

    av_frame.set_pts(Some((frame.pts.as_secs_f64() * 90000.0) as i64));
    av_frame.data_mut(0).copy_from_slice(&frame.data.y_plane);
    av_frame.data_mut(1).copy_from_slice(&frame.data.u_plane);
    av_frame.data_mut(2).copy_from_slice(&frame.data.v_plane);
    Ok(())
}
