use compositor_common::{scene::Resolution, Frame};
use crossbeam_channel::Sender;
use ffmpeg_next::{
    codec::{packet::Packet, Context, Id},
    format::Pixel,
    frame, Codec, Dictionary, Rational,
};
use log::{error, warn};
use serde::{Deserialize, Serialize};

use super::{OutputOptions, PipelineOutput};
use crate::error::OutputInitError;

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
    pub preset: EncoderPreset,
}

pub(crate) struct LibavH264Encoder {
    encoder: ffmpeg_next::codec::encoder::video::Encoder,
    codec: Codec,
    resolution: Resolution,
}

impl LibavH264Encoder {
    pub fn new(settings: EncoderSettings, resolution: Resolution) -> Result<Self, OutputInitError> {
        let codec = ffmpeg_next::codec::encoder::find(Id::H264).ok_or(OutputInitError::NoCodec)?;
        let mut encoder = Context::new().encoder().video()?;
        let pts_unit_secs = Rational::new(1, 90000);
        encoder.set_time_base(pts_unit_secs);
        encoder.set_format(Pixel::YUV420P);
        encoder.set_width(resolution.width as u32);
        encoder.set_height(resolution.height as u32);

        let encoder = encoder.open_as_with(
            codec,
            // TODO: audit settings bellow
            // Those values are copied from somewhere, they have to be set because libx264
            // is throwing an error if it detects default ffmpeg settings.
            Dictionary::from_iter([
                ("preset", settings.preset.to_str()),
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
                ("partitions", settings.preset.default_partitions()),
                // Subpixel motion estimation and mode decision (decision quality: 1=fast, 11=best)
                ("subq", settings.preset.default_subq_mode()),
            ]),
        )?;

        Ok(Self {
            encoder,
            codec,
            resolution,
        })
    }

    pub fn codec(&self) -> Codec {
        self.codec
    }

    pub fn send_frame(&mut self, frame: Frame) -> PacketIterator {
        let mut av_frame = frame::Video::new(
            Pixel::YUV420P,
            self.resolution.width as u32,
            self.resolution.height as u32,
        );

        frame_into_av(frame, &mut av_frame);

        if let Err(e) = self.encoder.send_frame(&av_frame) {
            error!("Encoder error: {e}.")
        }

        PacketIterator { encoder: self }
    }
}

pub struct PacketIterator<'a> {
    encoder: &'a mut LibavH264Encoder,
}

impl<'a> Iterator for PacketIterator<'a> {
    type Item = Packet;

    fn next(&mut self) -> Option<Self::Item> {
        let mut packet = Packet::empty();

        if self.encoder.encoder.receive_packet(&mut packet).is_ok() {
            Some(packet)
        } else {
            None
        }
    }
}

fn frame_into_av(frame: Frame, av_frame: &mut frame::Video) {
    let expected_y_plane_size = (av_frame.plane_width(0) * av_frame.plane_height(0)) as usize;
    let expected_u_plane_size = (av_frame.plane_width(1) * av_frame.plane_height(1)) as usize;
    let expected_v_plane_size = (av_frame.plane_width(2) * av_frame.plane_height(2)) as usize;
    if expected_y_plane_size != frame.data.y_plane.len() {
        error!(
            "Encoder: Y plane is a wrong size, expected: {} received: {}",
            expected_y_plane_size,
            frame.data.y_plane.len()
        );
        return;
    }
    if expected_u_plane_size != frame.data.u_plane.len() {
        error!(
            "Encoder: U plane is a wrong size, expected: {} received: {}",
            expected_u_plane_size,
            frame.data.u_plane.len()
        );

        return;
    }
    if expected_v_plane_size != frame.data.v_plane.len() {
        error!(
            "Encoder: V plane is a wrong size, expected: {} received: {}",
            expected_v_plane_size,
            frame.data.v_plane.len()
        );
        return;
    }

    av_frame.set_pts(Some((frame.pts.as_secs_f64() * 90000.0) as i64));

    write_plane_to_av(av_frame, 0, &frame.data.y_plane);
    write_plane_to_av(av_frame, 1, &frame.data.u_plane);
    write_plane_to_av(av_frame, 2, &frame.data.v_plane);
    // Ok(())
}

fn write_plane_to_av(frame: &mut frame::Video, plane: usize, data: &[u8]) {
    let stride = frame.stride(plane);
    let width = frame.plane_width(plane) as usize;

    data.chunks(width)
        .zip(frame.data_mut(plane).chunks_mut(stride))
        .for_each(|(data, target)| target[..width].copy_from_slice(data));
}

pub struct Encoder<Output: PipelineOutput> {
    sender: Sender<Frame>,
    output: Output,
}

impl<Output: PipelineOutput> Encoder<Output> {
    pub fn new(opts: OutputOptions<Output>) -> Result<Self, OutputInitError> {
        let mut encoder = LibavH264Encoder::new(opts.encoder_settings, opts.resolution)?;
        let (frame_sender, frame_receiver) = crossbeam_channel::unbounded();
        // channel used to return information about the RtpSender initialization back to the API thread.
        let (output_sender, output_receiver) = crossbeam_channel::bounded(0);

        std::thread::spawn(move || {
            let (output, mut context) = match Output::new(opts.receiver_options, encoder.codec()) {
                Ok(r) => r,
                Err(e) => {
                    output_sender.send(Err(e)).unwrap();
                    return;
                }
            };

            output_sender.send(Ok(output.clone())).unwrap();

            for frame in frame_receiver.iter() {
                if frame_receiver.len() > 20 {
                    warn!("Dropping frame: encoder queue is too long.");
                    continue;
                }

                for packet in encoder.send_frame(frame) {
                    output.send_packet(&mut context, packet);
                }
            }
        });

        Ok(Self {
            sender: frame_sender,
            output: output_receiver.recv().unwrap()?,
        })
    }

    pub fn send_frame(&self, frame: Frame) {
        self.sender.send(frame).unwrap();
    }

    pub fn output(&self) -> &Output {
        &self.output
    }
}
