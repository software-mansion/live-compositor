use anyhow::{anyhow, Result};
use crossbeam_channel::{Receiver, Sender};
use log::error;
use std::{path::PathBuf, thread};

use compositor_common::{scene::Resolution, Frame};
use compositor_pipeline::pipeline::PipelineOutput;
use ffmpeg_next::{
    codec::{self, Context, Id},
    format::{self, Pixel},
    frame, Dictionary, Rational,
};

pub struct RtpSender {
    #[allow(dead_code)]
    port: u16,
    sender: Sender<Frame>,
}

impl RtpSender {
    pub fn new(port: u16, resolution: Resolution) -> Self {
        let port_clone = port;
        let (sender, receiver) = crossbeam_channel::unbounded();
        thread::spawn(move || RtpSender::run(port_clone, resolution, receiver).unwrap());
        Self { port, sender }
    }

    fn run(port: u16, resolution: Resolution, receiver: Receiver<Frame>) -> Result<()> {
        let mut output_ctx =
            format::output_as(&PathBuf::from(format!("rtp://127.0.0.1:{}", port)), "rtp")?;
        let h264_codec = codec::encoder::find(Id::H264).unwrap();
        let mut stream = output_ctx.add_stream(h264_codec)?;
        unsafe {
            (*(*stream.as_mut_ptr()).codecpar).codec_id = codec::Id::H264.into();
        }

        let mut encoder = Context::new().encoder().video()?;
        let pts_unit_secs = Rational::new(1, 90000);
        encoder.set_time_base(pts_unit_secs);
        encoder.set_format(Pixel::YUV420P);
        encoder.set_width(resolution.width.try_into().unwrap());
        encoder.set_height(resolution.height.try_into().unwrap());

        let mut encoder = encoder.open_as_with(
            h264_codec,
            // TODO: audit settings bellow
            // Those values are copied from somewhere, they have to be set because libx264
            // is throwing an error if it detects default ffmpeg settings.
            Dictionary::from_iter([
                ("preset", "ultrafast"),
                ("tune", "zerolatency"),
                ("qcomp", "0.6"),
                ("i_qfactor", "0.71"),
                ("g", "250"),
                ("me_range", "16"),
                ("partitions", "+parti8x8+parti4x4+partp8x8+partb8x8"),
            ]),
        )?;
        output_ctx.write_header()?;
        let mut packet = codec::packet::Packet::empty();
        let mut av_frame = frame::Video::new(
            Pixel::YUV420P,
            resolution.width.try_into().unwrap(),
            resolution.height.try_into().unwrap(),
        );
        for frame in receiver.into_iter() {
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

    av_frame.set_pts(Some(
        ((frame.pts.as_nanos() as f64) * 90000.0 / 10e9) as i64,
    ));
    av_frame.data_mut(0).copy_from_slice(&frame.data.y_plane);
    av_frame.data_mut(1).copy_from_slice(&frame.data.u_plane);
    av_frame.data_mut(2).copy_from_slice(&frame.data.v_plane);
    Ok(())
}
