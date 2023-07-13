use anyhow::{anyhow, Result};
use compositor_common::{frame::YuvData, scene::Resolution, Frame};
use std::{fs::File, io::Write, path::PathBuf, sync::Arc, thread};

use ffmpeg_next::{
    codec::{self, Context, Id},
    format,
    frame::{self, Video},
    media::Type,
    Dictionary,
};

use crate::state::Pipeline;

#[allow(dead_code)]
pub struct RtpReceiver {
    port: u16,
}

impl RtpReceiver {
    pub fn new(pipeline: Arc<Pipeline>, port: u16) -> Self {
        let port_clone = port;
        thread::spawn(move || {
            RtpReceiver::start(pipeline, port_clone).unwrap();
        });
        Self { port }
    }

    fn start(pipeline: Arc<Pipeline>, port: u16) -> Result<()> {
        let sdp_filepath = PathBuf::from(format!("/tmp/sdp_input_{}.sdp", port));
        let mut file = File::create(&sdp_filepath)?;
        file.write_all(
            format!(
                "\
                    v=0\n\
                    o=- 0 0 IN IP4 127.0.0.1\n\
                    s=No Name\n\
                    c=IN IP4 127.0.0.1\n\
                    m=video {} RTP/AVP 96\n\
                    a=rtpmap:96 H264/90000\n\
                    a=fmtp:96 packetization-mode=1\n\
                    a=rtcp-mux\n\
                ",
                port
            )
            .as_bytes(),
        )?;
        let mut input_ctx = format::input_with_dictionary(
            &sdp_filepath,
            Dictionary::from_iter([("protocol_whitelist", "file,udp,rtp")]),
        )?;
        let input = input_ctx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg_next::Error::StreamNotFound)?;
        let input_index = input.index();
        let decoder = Context::from_parameters(input.parameters())?;
        let decoder = decoder.decoder();
        let h264_codec = codec::decoder::find(Id::H264).unwrap();
        let mut decoder = decoder.open_as(h264_codec)?;

        let mut pts_offset: Option<i64> = None;
        let mut decoded_frame = frame::Video::empty();
        for (stream, packet) in input_ctx.packets() {
            if stream.index() != input_index {
                eprintln!("Received frame from unknown stream, skipping");
                continue;
            }

            decoder.send_packet(&packet)?;
            while decoder.receive_frame(&mut decoded_frame).is_ok() {
                let frame = match frame_from_av(&mut decoded_frame, &mut pts_offset) {
                    Ok(frame) => frame,
                    Err(err) => {
                        eprintln!("Dropping frame: {err}");
                        continue;
                    }
                };
                pipeline.push_input_data(port.into(), frame);
            }
        }

        Ok(())
    }
}

fn frame_from_av(decoded: &mut Video, pts_offset: &mut Option<i64>) -> Result<Frame> {
    if decoded.format() != format::pixel::Pixel::YUV420P {
        return Err(anyhow!("only YUV420P is supported"));
    }
    let original_pts = decoded.pts();
    if let (Some(pts), None) = (decoded.pts(), &pts_offset) {
        *pts_offset = Some(-pts)
    }
    let pts = original_pts.map(|original_pts| original_pts + pts_offset.unwrap_or(0));
    Ok(Frame {
        data: YuvData {
            y_plane: bytes::Bytes::copy_from_slice(decoded.data(0)),
            u_plane: bytes::Bytes::copy_from_slice(decoded.data(1)),
            v_plane: bytes::Bytes::copy_from_slice(decoded.data(2)),
        },
        resolution: Resolution {
            width: decoded.width().try_into()?,
            height: decoded.height().try_into()?,
        },
        pts: pts.ok_or_else(|| anyhow!("missing pts"))?,
    })
}
